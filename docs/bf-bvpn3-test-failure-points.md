# Test Failure Points Documentation
**Bead:** bf-bvpn3  
**Date:** 2026-08-02  
**Parent Bead:** bf-3thju (Failing Tests Catalog)  
**Source:** bf-7vowz-verification.md, bf-2amik-verification.md, bf-3thju-failing-tests-catalog.md

---

## Overview

This document maps each failing test to its exact failure point within the test lifecycle. All 9 tests fail **before runtime execution** - either at compilation or are blocked from compilation entirely.

**Failure Distribution:**
- **Pre-compilation (blocked):** 3 tests - never compiled due to sibling file errors
- **Compilation phase:** 6 tests - failed with specific compiler errors at specific lines

---

## Category 1: Blocked Tests (Pre-compilation)

### Test 1: `test_beads_deletion_readyz_degraded`
**File:** `hoop-daemon/tests/beads_deletion_http.rs`  
**Purpose:** Verify `/readyz` endpoint reports degraded state when `.beads` directory is deleted

**Failure Point:** **Pre-compilation (BLOCKED)**  
**Status:** Never reached compilation phase  
**Blocking Errors:** Compilation failures in `property_invariants.rs` and `draft_queue_invariants.rs`

**Lifecycle Stage:** SETUP → **BLOCKED**  
**Details:**
- Test file compiles successfully in isolation
- Cannot execute because `cargo test` compiles entire test target first
- Sibling file errors prevent compilation of the entire test target
- No test code ever executed

---

### Test 2: `test_beads_deletion_sibling_events_continue`
**File:** `hoop-daemon/tests/beads_deletion_http.rs`  
**Purpose:** Verify sibling projects continue serving WebSocket events during one project's `.beads` deletion

**Failure Point:** **Pre-compilation (BLOCKED)**  
**Status:** Never reached compilation phase  
**Blocking Errors:** Compilation failures in `property_invariants.rs` and `draft_queue_invariants.rs`

**Lifecycle Stage:** SETUP → **BLOCKED**  
**Details:**
- Test file compiles successfully in isolation
- Cannot execute because `cargo test` compiles entire test target first
- Sibling file errors prevent compilation of the entire test target
- No test code ever executed

---

### Test 3: `test_readyz_response_format`
**File:** `hoop-daemon/tests/beads_deletion_http.rs`  
**Purpose:** Verify `/readyz` response JSON format matches expected schema

**Failure Point:** **Pre-compilation (BLOCKED)**  
**Status:** Never reached compilation phase  
**Blocking Errors:** Compilation failures in `property_invariants.rs` and `draft_queue_invariants.rs`

**Lifecycle Stage:** SETUP → **BLOCKED**  
**Details:**
- Test file compiles successfully in isolation
- Cannot execute because `cargo test` compiles entire test target first
- Sibling file errors prevent compilation of the entire test target
- No test code ever executed

---

## Category 2: Property-Based Invariant Tests (Compilation Failures)

### Test 4: `proptest_replay_equals_live_inner`
**File:** `hoop-daemon/tests/property_invariants.rs` (line 654)  
**Purpose:** Verify event replay produces identical state to live event processing

**Failure Point:** **Compilation - Test Setup (Proptest Strategy Generation)**  
**Error Codes:** E0434 (1x), E0433 (4x)  
**Failure Lines:** 657, 670, 678, 828, 838

**Lifecycle Stage:** SETUP → **COMPILATION ERROR**  
**Specific Failures:**

1. **Line 657 - E0434:** Proptest strategy attempts to capture dynamic environment
   ```rust
   events in prop::collection::vec(event_strategy, 0..20)
   //       ^^^^^^^^^^^^^^ - needs || { ... } closure form
   ```

2. **Line 670 - E0433:** Missing `File` type import
   ```rust
   let mut file = File::create(&events_path).unwrap();
   //               ^^^^ - undeclared type
   ```

3. **Line 678 - E0433:** Missing `File` type import
   ```rust
   let file = File::open(&events_path).unwrap();
   //           ^^^^ - undeclared type
   ```

4. **Lines 828, 838 - E0433:** Additional missing `File` imports in idempotent test

**Why Setup Fails:** Test cannot generate input strategies without fixing proptest syntax and importing `File` type

---

### Test 5: `proptest_replay_handles_partial_lines_inner`
**File:** `hoop-daemon/tests/property_invariants.rs` (line 729)  
**Purpose:** Verify event replay correctly handles split/partial JSON lines

**Failure Point:** **Compilation - Test Setup (Proptest Strategy Generation)**  
**Error Codes:** E0434 (3x)  
**Failure Lines:** 732, 735, 736, 745

**Lifecycle Stage:** SETUP → **COMPILATION ERROR**  
**Specific Failures:**

1. **Line 732 - E0434:** Strategy captures dynamic environment
   ```rust
   split_pos in 0..valid_event.len()
   //           ^^^^^^^^^^^ - needs closure form
   ```

2. **Line 735 - E0434:** Variable capture in strategy
   ```rust
   let chunk1 = &valid_event[..split_pos];
   //             ^^^^^^^^^^^ - needs closure form
   ```

3. **Line 736 - E0434:** Variable capture in strategy
   ```rust
   let chunk2 = &valid_event[split_pos..];
   //             ^^^^^^^^^^^ - needs closure form
   ```

4. **Line 745 - E0434:** Variable capture in strategy
   ```rust
   let split_at_boundary = split_pos == 0 || split_pos == valid_event.len();
   //                                                             ^^^^^^^^^^^ - needs closure form
   ```

**Why Setup Fails:** Test cannot generate split positions without closure-form proptest strategies

---

### Test 6: `proptest_replay_is_idempotent_inner`
**File:** `hoop-daemon/tests/property_invariants.rs` (line 818)  
**Purpose:** Verify event replay is idempotent (multiple replays produce same result)

**Failure Point:** **Compilation - Test Setup (Proptest Strategy Generation)**  
**Error Codes:** E0434 (1x), E0433 (5x)  
**Failure Lines:** 821, 828, 838, 867, 874, 881, 887

**Lifecycle Stage:** SETUP → **COMPILATION ERROR**  
**Specific Failures:**

1. **Line 821 - E0434:** Proptest strategy captures dynamic environment
   ```rust
   events in prop::collection::vec(event_strategy, 0..10)
   //       ^^^^^^^^^^^^^^ - needs closure form
   ```

2. **Lines 828, 838, 867, 874, 881, 887 - E0433:** Missing `File` type imports (6 occurrences)
   ```rust
   let mut file = File::create(&events_path).unwrap();  // 828, 867, 881
   let file = File::open(&events_path).unwrap();        // 838, 874, 887
   //           ^^^^ - undeclared type
   ```

**Why Setup Fails:** Test cannot create test fixtures without importing `File` type

---

### Test 7: `test_stitch_status_purity`
**File:** `hoop-daemon/tests/property_invariants.rs` (line 380)  
**Purpose:** Verify `derive_status()` is deterministic/pure function

**Failure Point:** **Compilation - Test Assertion (Type System Error)**  
**Error Codes:** E0277 (1x), E0382 (1x)  
**Failure Lines:** 380, 576

**Lifecycle Stage:** ASSERTION → **COMPILATION ERROR**  
**Specific Failures:**

1. **Line 576 - E0277:** Missing trait implementation
   ```rust
   results.insert(ctx.derive_status());
   //            ^^^^^^ - StitchStatus doesn't implement Hash
   ```

2. **Line 380 - E0382:** Ownership error
   ```rust
   prop_assert_eq!(status1, status2, "First and second calls differ");
   //                                 ------- value moved here
   prop_assert_eq!(status2, status3, "Second and third calls differ");
   //                 ^^^^^^^ value used here after move
   ```

**Why Assertions Fail:** Test attempts to use `HashSet` for deduplication but `StitchStatus` lacks `Hash` trait; also tries to reuse moved value

---

## Category 3: Draft Queue Invariant Tests (Compilation Failures)

### Test 8: `test_draft_preview_flow`
**File:** `hoop-daemon/tests/draft_queue_invariants.rs` (line 363)  
**Purpose:** Verify draft preview queue operations work correctly

**Failure Point:** **Compilation - Test Setup (Struct Initialization)**  
**Error Code:** E0063  
**Failure Line:** 363

**Lifecycle Stage:** SETUP → **COMPILATION ERROR**  
**Specific Failure:**

1. **Line 363 - E0063:** Missing struct fields
   ```rust
   let draft = hoop_daemon::fleet::DraftRow {
   //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing 5 fields
   ```

**Missing Fields:** `abandoned_at`, `last_autosave_at`, `opened_at`, plus 2 others  
**Why Setup Fails:** Production struct `DraftRow` gained new fields but test fixture initializer was never updated

---

### Test 9: `test_draft_abandon_timeout`
**File:** `hoop-daemon/tests/draft_queue_invariants.rs` (line 506)  
**Purpose:** Verify drafts auto-abandon after timeout period

**Failure Point:** **Compilation - Test Setup (Struct Initialization)**  
**Error Code:** E0063  
**Failure Line:** 506

**Lifecycle Stage:** SETUP → **COMPILATION ERROR**  
**Specific Failure:**

1. **Line 506 - E0063:** Missing struct fields
   ```rust
   let draft = hoop_daemon::fleet::DraftRow {
   //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing 5 fields
   ```

**Missing Fields:** `abandoned_at`, `last_autosave_at`, `opened_at`, plus 2 others (same as Test 8)  
**Why Setup Fails:** Production struct `DraftRow` gained new fields but test fixture initializer was never updated

---

## Additional Compilation Errors (Not Test-Specific)

### Error: Missing `BufRead` Trait
**Line:** 258 in `property_invariants.rs`  
**Error Code:** E0599  
**Context:** Test helper function, not specific to one test

```rust
for line in reader.lines() {
//               ^^^^^ - method not found without BufRead trait
```

**Fix Required:** Add `use std::io::BufRead;` to imports

---

## Test Lifecycle Failure Mapping

### Test Lifecycle Stages
1. **SETUP** - Test initialization, fixture creation, strategy generation
2. **READINESS** - Pre-test checks (service up, dependencies available)
3. **EXECUTION** - Main test logic runs
4. **ASSERTION** - Verification of expected outcomes
5. **CLEANUP** - Resource release, teardown

### Failure Distribution by Stage

| Stage | Count | Tests |
|-------|-------|-------|
| Pre-compilation (blocked) | 3 | All `beads_deletion_http` tests |
| Setup (compilation) | 5 | Tests 4, 5, 6, 8, 9 |
| Assertion (compilation) | 1 | Test 7 (`test_stitch_status_purity`) |
| Execution (runtime) | 0 | No tests reached execution |
| Cleanup (runtime) | 0 | No tests reached cleanup |

### Key Finding: **No Runtime Failures**

**100% of failures occur at compile-time.** No test reached:
- Readiness checks (service startup, dependency availability)
- Test execution (actual test logic running)
- Assertion verification (checking expected vs actual)
- Cleanup (resource teardown)

---

## Error Frequency by Type

| Error Code | Count | Description | Failure Stage |
|------------|-------|-------------|---------------|
| E0434 | 6 | Proptest captures dynamic environment | Setup |
| E0433 | 9 | Missing `File` type import | Setup |
| E0063 | 2 | Missing struct fields | Setup |
| E0599 | 1 | Missing `BufRead` trait | Setup |
| E0277 | 1 | Missing `Hash` trait | Assertion |
| E0382 | 1 | Use of moved value | Assertion |
| **Total** | **21** | | |

---

## Test-to-Failure-Point Matrix

| Test ID | Test Name | File | Failure Stage | Error Code | Line |
|---------|-----------|------|---------------|------------|------|
| 1 | `test_beads_deletion_readyz_degraded` | beads_deletion_http.rs | Pre-compilation (blocked) | N/A | N/A |
| 2 | `test_beads_deletion_sibling_events_continue` | beads_deletion_http.rs | Pre-compilation (blocked) | N/A | N/A |
| 3 | `test_readyz_response_format` | beads_deletion_http.rs | Pre-compilation (blocked) | N/A | N/A |
| 4 | `proptest_replay_equals_live_inner` | property_invariants.rs | Setup (compilation) | E0434, E0433 | 657, 670, 678 |
| 5 | `proptest_replay_handles_partial_lines_inner` | property_invariants.rs | Setup (compilation) | E0434 | 732, 735, 736 |
| 6 | `proptest_replay_is_idempotent_inner` | property_invariants.rs | Setup (compilation) | E0434, E0433 | 821, 828, 838 |
| 7 | `test_stitch_status_purity` | property_invariants.rs | Assertion (compilation) | E0277, E0382 | 380, 576 |
| 8 | `test_draft_preview_flow` | draft_queue_invariants.rs | Setup (compilation) | E0063 | 363 |
| 9 | `test_draft_abandon_timeout` | draft_queue_invariants.rs | Setup (compilation) | E0063 | 506 |

---

## Failure Point Classification Summary

### By Test Lifecycle Stage
- **Pre-compilation (blocked):** 33% (3/9) - Target tests never compiled
- **Setup (compilation):** 56% (5/9) - Test fixture/strategy generation failed
- **Assertion (compilation):** 11% (1/9) - Type system error in assertions

### By Error Category
- **Import errors:** 48% (10/21) - Missing `File` and `BufRead` imports
- **Proptest syntax errors:** 29% (6/21) - Closure form needed
- **Struct drift errors:** 10% (2/21) - Production gained fields, tests not updated
- **Type system errors:** 10% (2/21) - Missing traits, ownership issues
- **Method resolution:** 5% (1/21) - Missing trait for method call

### Determinism Classification
- **Deterministic:** 100% (9/9) - All failures are compilation errors
- **Flaky:** 0% - No timing, race conditions, or environment-dependent failures

---

## Recommendations by Failure Stage

### Stage 1: Fix Imports (Quick Wins - ~10 minutes)
1. Add `use std::fs::File;` to `property_invariants.rs` (fixes 9 errors)
2. Add `use std::io::BufRead;` to `property_invariants.rs` (fixes 1 error)

### Stage 2: Fix Proptest Syntax (~30 minutes)
3. Convert 6 proptest strategies to closure form: `|| { ... }` (fixes 6 errors)

### Stage 3: Fix Type System (~20 minutes)
4. Add `#[derive(Hash)]` to `StitchStatus` OR use `Vec` instead of `HashSet` (fixes 1 error)
5. Add `.clone()` to line 379 (fixes 1 error)

### Stage 4: Fix Struct Drift (~30 minutes)
6. Update `DraftRow` initializers with 5 new fields in both test fixtures (fixes 2 errors)

**Total Estimated Effort:** 1.5-2 hours

---

## Next Steps

1. **Apply fixes** in priority order above
2. **Re-run compilation:** `cargo test --package hoop-daemon --lib`
3. **Observe runtime behavior** of the 3 blocked `beads_deletion_http` tests
4. **Document runtime failure points** if any appear (update this file)
5. **Classify runtime failures** as flaky vs deterministic

---

## Appendix: Source Reference

- **Raw Output:** `/home/coding/HOOP/bf-7vowz-verification.md`
- **Individual Analysis:** `/home/coding/HOOP/bf-2amik-verification.md`
- **Test Catalog:** `/home/coding/HOOP/docs/bf-3thju-failing-tests-catalog.md`
- **Parent Bead:** bf-3thju (Extract and catalog failing test names)
- **Grandparent Bead:** bf-2amik (Individual test failure pattern analysis)
- **Great-Grandparent Bead:** bf-2p2cr (Complete test failure analysis)

---

**End of Failure Points Documentation**
