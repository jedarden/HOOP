# Failing Tests Catalog
**Bead:** bf-3thju  
**Date:** 2026-08-02  
**Source:** bf-7vowz-verification.md, bf-2amik-verification.md  
**Parent Bead:** bf-2amik  

---

## Overview

This catalog extracts and categorizes all failing tests from the HOOP test suite execution. The test run encountered **compilation failures** that prevented the target `beads_deletion_http` tests from executing, while also revealing compilation errors in other test files.

**Total Affected Tests:** 9 tests across 3 test files
**Breakdown:**
- 3 tests BLOCKED from execution (intended target tests)
- 6 tests FAILED at compilation (property and draft invariant tests)

---

## Category 1: Blocked Tests (Intended Target)

These tests were the primary target of the test run but were **blocked from execution** due to compilation failures in sibling test files. Rust's `cargo test` compiles all tests in a target before running any, so errors in unrelated files prevented execution.

### File: `hoop-daemon/tests/beads_deletion_http.rs`

#### 1. `test_beads_deletion_readyz_degraded`
- **Purpose:** Verify `/readyz` endpoint reports degraded state when `.beads` directory is deleted
- **Expected Behavior:** 
  - Delete `.beads` directory from a test project
  - Call `/readyz` health endpoint
  - Verify response indicates "degraded" status
  - Confirm service remains partially functional
- **Failure Point:** Pre-compilation (never reached execution)
- **Blocking Error:** Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs`
- **Status:** BLOCKED
- **Error Type:** N/A (blocked by sibling file compilation failures)

#### 2. `test_beads_deletion_sibling_events_continue`
- **Purpose:** Verify sibling projects continue serving WebSocket events during one project's `.beads` deletion
- **Expected Behavior:**
  - Create multiple test projects (siblings)
  - Delete `.beads` from one project
  - Verify other projects' WebSocket event streams continue uninterrupted
  - Confirm isolation between projects
- **Failure Point:** Pre-compilation (never reached execution)
- **Blocking Error:** Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs`
- **Status:** BLOCKED
- **Error Type:** N/A (blocked by sibling file compilation failures)

#### 3. `test_readyz_response_format`
- **Purpose:** Verify `/readyz` response JSON format matches expected schema
- **Expected Behavior:**
  - Call `/readyz` endpoint
  - Verify JSON response structure matches schema
  - Check required fields: status, projects, degraded_projects
  - Validate field types and value constraints
- **Failure Point:** Pre-compilation (never reached execution)
- **Blocking Error:** Compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs`
- **Status:** BLOCKED
- **Error Type:** N/A (blocked by sibling file compilation failures)

---

## Category 2: Property-Based Invariant Tests (Compilation Failures)

These tests use proptest for property-based testing but have compilation errors that block the entire test suite.

### File: `hoop-daemon/tests/property_invariants.rs`

#### 4. `proptest_replay_equals_live_inner`
- **Purpose:** Verify event replay produces identical state to live event processing
- **Property:** For any event stream, replaying from disk produces the same internal state
- **Expected Behavior:**
  - Generate random event sequences
  - Process events live
  - Write events to disk
  - Replay from disk
  - Verify states match
- **Failure Point:** Compilation (line 657)
- **Error Codes:** E0434 (1x), E0433 (4x)
- **Root Causes:**
  - Proptest strategy attempts to capture dynamic environment (needs `|| { ... }` closure)
  - Missing `use std::fs::File;` import (4 occurrences)
- **Status:** FAILED (compilation error)

#### 5. `proptest_replay_handles_partial_lines_inner`
- **Purpose:** Verify event replay correctly handles split/partial JSON lines
- **Property:** Event replay handles incomplete reads gracefully (e.g., when events span read boundaries)
- **Expected Behavior:**
  - Create valid event JSON
  - Split at arbitrary byte positions
  - Verify replay parses partial lines correctly
  - Confirm no data loss or corruption
- **Failure Point:** Compilation (line 732)
- **Error Codes:** E0434 (3x)
- **Root Causes:**
  - Proptest strategy captures dynamic environment variables (needs closure form at lines 732, 735, 736, 745)
- **Status:** FAILED (compilation error)

#### 6. `proptest_replay_is_idempotent_inner`
- **Purpose:** Verify event replay is idempotent (multiple replays produce same result)
- **Property:** Replaying the same event file twice produces identical state
- **Expected Behavior:**
  - Generate random event sequences
  - Write to disk
  - Replay twice
  - Verify both replay results match
- **Failure Point:** Compilation (line 821)
- **Error Codes:** E0434 (1x), E0433 (5x)
- **Root Causes:**
  - Proptest strategy captures dynamic environment (needs closure form)
  - Missing `use std::fs::File;` import (5 occurrences)
- **Status:** FAILED (compilation error)

#### 7. `test_stitch_status_purity`
- **Purpose:** Verify `derive_status()` is deterministic/pure function
- **Property:** Given same `StitchContext`, `derive_status()` always returns same `StitchStatus`
- **Expected Behavior:**
  - Create `StitchContext` with fixed state
  - Call `derive_status()` three times
  - Verify all results equal (insert into `HashSet` for deduplication)
  - Confirm no side effects or randomness
- **Failure Point:** Compilation (lines 380, 576)
- **Error Codes:** E0277 (1x), E0382 (1x)
- **Root Causes:**
  - `StitchStatus` doesn't implement `Hash` trait needed for `HashSet` (line 576)
  - Use of moved value `status2` after first comparison (line 380, needs `.clone()`)
- **Status:** FAILED (compilation error)

---

## Category 3: Draft Queue Invariant Tests (Compilation Failures)

These tests verify draft queue behavior but have struct field drift issues.

### File: `hoop-daemon/tests/draft_queue_invariants.rs`

#### 8. `test_draft_preview_flow`
- **Purpose:** Verify draft preview queue operations work correctly
- **Expected Behavior:**
  - Create draft in preview queue
  - Simulate user edits and autosave
  - Verify draft state transitions correctly
  - Confirm queue operations are atomic
- **Failure Point:** Compilation (line 363)
- **Error Code:** E0063
- **Root Cause:** `DraftRow` struct initializer missing 5 new fields: `abandoned_at`, `last_autosave_at`, `opened_at`, plus 2 others
- **Impact:** Production struct gained fields but test fixtures were never updated
- **Status:** FAILED (compilation error)

#### 9. `test_draft_abandon_timeout`
- **Purpose:** Verify drafts auto-abandon after timeout period
- **Expected Behavior:**
  - Create draft with timestamp
  - Advance time past timeout threshold
  - Verify draft auto-abandons
  - Confirm abandoned drafts don't block queue
- **Failure Point:** Compilation (line 506)
- **Error Code:** E0063
- **Root Cause:** `DraftRow` struct initializer missing 5 new fields (same as above)
- **Impact:** Production struct gained fields but test fixtures were never updated
- **Status:** FAILED (compilation error)

---

## Compilation Error Summary

### Error Frequency by Type

| Error Code | Count | Description | Files Affected |
|------------|-------|-------------|----------------|
| E0434 | 6 | Proptest strategy captures dynamic environment | property_invariants.rs |
| E0433 | 9 | Missing `File` type import | property_invariants.rs |
| E0063 | 2 | Missing struct fields in initializer | draft_queue_invariants.rs |
| E0599 | 1 | Missing `BufRead` trait for `.lines()` method | property_invariants.rs |
| E0277 | 1 | Missing `Hash` trait implementation | property_invariants.rs |
| E0382 | 1 | Use of moved value (needs clone) | property_invariants.rs |
| **Total** | **21** | | |

### Error Distribution by File

| File | Errors | Tests Blocked |
|------|--------|---------------|
| `property_invariants.rs` | 19 | 4 tests |
| `draft_queue_invariants.rs` | 2 | 2 tests |
| **Total** | **21** | **6 tests + 3 blocked tests** |

---

## Test Functionality Mapping

### Health Endpoint Tests (beads_deletion_http.rs)
- ✗ `test_beads_deletion_readyz_degraded` - Degraded state detection
- ✗ `test_beads_deletion_sibling_events_continue` - Multi-project isolation  
- ✗ `test_readyz_response_format` - Response schema validation

### Event Replay Invariants (property_invariants.rs)
- ✗ `proptest_replay_equals_live_inner` - Replay correctness
- ✗ `proptest_replay_handles_partial_lines_inner` - Partial line handling
- ✗ `proptest_replay_is_idempotent_inner` - Replay idempotency

### Stitch Status Invariants (property_invariants.rs)
- ✗ `test_stitch_status_purity` - Deterministic status derivation

### Draft Queue Invariants (draft_queue_invariants.rs)
- ✗ `test_draft_preview_flow` - Preview queue operations
- ✗ `test_draft_abandon_timeout` - Auto-abandon timeout behavior

---

## Failure Mode Classification

### Deterministic Failures (100%)

All 9 test failures are **deterministic**:
- **Compilation errors (6 tests):** Compiler errors always fail on same source
- **Blocked tests (3 tests):** Deterministically blocked by sibling compilation failures

**No flakiness detected.** No timing, race conditions, or environment-dependent failures.

### Failure Phase Distribution

| Phase | Count | Tests |
|-------|-------|-------|
| Pre-compilation (blocked) | 3 | All beads_deletion_http tests |
| Compilation | 6 | Property + draft invariant tests |
| Runtime | 0 | No tests reached execution |

---

## Recommended Fix Priority

### Phase 1: Quick Wins (Single-line fixes, ~10 minutes)
1. Add missing imports to `property_invariants.rs`:
   - `use std::fs::File;` (fixes 9 errors)
   - `use std::io::BufRead;` (fixes 1 error)
2. Fix ownership error: Add `.clone()` to line 379 (fixes 1 error)

### Phase 2: Proptest Syntax (~30 minutes)
3. Convert 6 proptest strategies to closure form: `|| { ... }` (fixes 6 errors)

### Phase 3: Type System (~20 minutes)
4. Add `#[derive(Hash)]` to `StitchStatus` OR use `Vec` instead of `HashSet` (fixes 1 error)

### Phase 4: Struct Field Updates (~30 minutes)
5. Update `DraftRow` initializers with 5 new fields (fixes 2 errors)

**Total Estimated Effort:** 1.5-2 hours

---

## Next Steps

1. **Fix compilation errors** in priority order above
2. **Re-run test suite:** `cargo test --package hoop-daemon --lib`
3. **Observe runtime behavior** of the 3 blocked `beads_deletion_http` tests
4. **Classify any runtime failures** as flaky vs deterministic
5. **Update this catalog** with runtime failure patterns if they appear

---

## Appendix: Source Reference

- **Raw Output:** `/home/coding/HOOP/bf-7vowz-verification.md`
- **Analysis:** `/home/coding/HOOP/bf-2amik-verification.md`
- **Parent Bead:** bf-2amik (Individual test failure pattern analysis)
- **Grandparent Bead:** bf-2p2cr (Complete test failure analysis)

---

**End of Catalog**