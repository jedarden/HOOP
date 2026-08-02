# Failing Tests Catalog - beads_deletion_http

**Source:** Bead bf-7vowz test run  
**Generated:** 2026-08-02  
**Test Suite:** `hoop-daemon/tests/beads_deletion_http.rs`

## Executive Summary

The `beads_deletion_http` tests **did not execute** due to compilation failures in the test suite. This document catalogs the intended tests that were blocked from running.

**Compilation Blocker:** 21 errors across 2 test files (`property_invariants.rs`, `draft_queue_invariants.rs`) prevented the entire test suite from compiling.

---

## Intended Tests (Blocked from Execution)

### 1. `test_beads_deletion_readyz_degraded`

**Purpose:** Verify that the `/readyz` health endpoint reports degraded status when the `.beads` directory is deleted.

**What it tests:**
- Health check endpoint behavior under data loss conditions
- Graceful degradation when the bead queue becomes unavailable
- `/readyz` endpoint response to missing `.beads/` directory

**Expected behavior:**
- Endpoint should return `503 Service Unavailable` or degraded status
- System should detect missing bead queue and report appropriately
- No crash or panic when `.beads/` is missing

---

### 2. `test_beads_deletion_sibling_events_continue`

**Purpose:** Verify that sibling projects continue to serve events when one project's `.beads` directory is deleted.

**What it tests:**
- Multi-project isolation under degradation conditions
- Event serving isolation between projects
- Fault containment — one project's data loss doesn't affect others

**Expected behavior:**
- Sibling projects (with intact `.beads/`) continue operating normally
- Only the affected project shows degraded status
- Event streaming continues for unaffected projects

---

### 3. `test_readyz_response_format`

**Purpose:** Verify that the `/readyz` response format matches the expected schema.

**What it tests:**
- HTTP response structure (headers, status code, content type)
- JSON body schema validation
- Field presence and types in health check response

**Expected behavior:**
- Response includes required fields (status, timestamp, project states)
- Content-Type is `application/json`
- Status field is one of: `ok`, `degraded`, `down`

---

## Compilation Failures (Blocking Root Cause)

### Files with Errors

1. **`property_invariants.rs`** — 19 errors
   - Missing `File` imports (9x)
   - Proptest strategy closure issues (6x)
   - Missing `BufRead` trait import (1x)
   - `StitchStatus` missing `Hash` trait (1x)
   - Use-after-move error (1x)

2. **`draft_queue_invariants.rs`** — 2 errors
   - `DraftRow` struct missing 5 fields (2x): `abandoned_at`, `last_autosave_at`, `opened_at`, and 2 others

### Impact

Since `cargo test` compiles all tests in the `hoop-daemon` test target before running any, compilation errors in **unrelated test files** blocked execution of the target `beads_deletion_http` tests.

---

## Next Steps

1. Fix compilation errors in `property_invariants.rs` and `draft_queue_invariants.rs`
2. Re-run `beads_deletion_http` tests to capture actual execution results
3. Analyze individual test failure patterns once tests execute
4. Verify fix effectiveness against documented failure scenarios

---

## Related Beads

- **bf-7vowz** — Run tests and capture output (CLOSED)
- **bf-4l8jp** — Extract raw output per test (CLOSED)
- **bf-2amik** — Analyze failure patterns (BLOCKED, awaiting compilation fixes)
- **bf-3thju** — This bead (IN_PROGRESS)

