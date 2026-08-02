# Test Classification: Deterministic vs. Flaky Analysis

**Bead ID:** bf-517jz
**Analysis Date:** 2026-08-02
**Source Beads:** bf-4l8jp (raw output), bf-wss0q (failure documentation), bf-236t4 (initial classification)

## Executive Summary

**ALL THREE `beads_deletion_http` tests are DETERMINISTIC FAILURES**

| Test Name | Classification | Confidence | Failure Point |
|-----------|---------------|------------|---------------|
| `test_beads_deletion_readyz_degraded` | **DETERMINISTIC** | 100% | Compilation block |
| `test_beads_deletion_sibling_events_continue` | **DETERMINISTIC** | 100% | Compilation block |
| `test_readyz_response_format` | **DETERMINISTIC** | 100% | Compilation block |

**Key Finding:** None of these tests exhibit flaky behavior. All failures occur at compilation time, before any test code executes. This is a **deterministic compilation blocker**, not a runtime flakiness issue.

---

## Classification Framework

### Deterministic Failure Criteria
A test is **deterministic** when it:
- Fails consistently with the same error on every run
- Failure is not timing-dependent
- Failure is not environment-dependent (load, timing, concurrency)
- Root cause is reproducible 100% of the time

### Flaky Failure Criteria
A test is **flaky** when it:
- Fails intermittently (passes sometimes, fails other times)
- Failure depends on timing (race conditions, timeouts)
- Failure depends on resource availability (ports, disk space, memory)
- Failure depends on concurrent execution order
- Root cause is non-deterministic or environment-specific

---

## Test-by-Test Analysis

### Test 1: `test_beads_deletion_readyz_degraded`

#### Classification: ✅ DETERMINISTIC FAILURE

**Confidence:** 100%

**Failure Point:** Compilation Phase - Test never reached execution

**Rationale:**

1. **Consistency of Failure:** 100% reproducible
   - The test fails at compilation time, before any test code runs
   - Compilation errors are deterministic - the same 21 errors occur every time
   - No variation in error type, location, or message across runs

2. **No Timing Dependence:**
   - Failure occurs during Rust's static analysis phase
   - No runtime execution, no timeouts, no race conditions
   - Compilation order is deterministic within the test target

3. **No Resource Dependence:**
   - Not dependent on available ports, disk space, or memory at runtime
   - Only depends on source code correctness, which is invariant
   - No external services or networks involved

4. **No Concurrency Issues:**
   - Test never reached multi-threaded execution
   - Failure occurs in single-threaded compiler frontend
   - No parallel test execution possible (compilation blocks first)

**Blocking Compilation Errors (21 total):**
- 6 proptest strategy capture errors (property_invariants.rs)
- 9 missing `std::fs::File` imports (property_invariants.rs)
- 1 missing `std::io::BufRead` trait (property_invariants.rs)
- 1 missing `Hash` trait on `StitchStatus` (property_invariants.rs)
- 1 use-after-move error (property_invariants.rs)
- 2 missing struct fields in `DraftRow` (draft_queue_invariants.rs)

**Why Not Flaky:**
- Flaky tests require execution to exhibit intermittent behavior
- These tests never execute - they're rejected at the compilation gate
- Compiler errors are all-or-nothing: either code compiles or it doesn't
- No environment variable, timing, or load factor affects compilation outcome

**What Would Make This Flaky (Hypothetical):**
If the test compiled and ran, potential flakiness could arise from:
- Race conditions in daemon startup detection
- Timeouts in waiting for degraded state to propagate
- Filesystem timing in `.beads/` deletion detection
- Concurrent HTTP requests during state transition

**BUT: These are hypothetical only - test never reaches execution.**

---

### Test 2: `test_beads_deletion_sibling_events_continue`

#### Classification: ✅ DETERMINISTIC FAILURE

**Confidence:** 100%

**Failure Point:** Compilation Phase - Test never reached execution

**Rationale:**

1. **Consistency of Failure:** 100% reproducible
   - Same compilation blocker as Test 1
   - Fails for identical reasons - same 21 compilation errors
   - Cannot pass until underlying compilation errors are fixed

2. **No Timing Dependence:**
   - Blocked at compilation, no runtime behavior
   - No timeouts, no delays, no async operations

3. **No Resource Dependence:**
   - Not dependent on runtime resource state
   - Only dependent on source code correctness

4. **No Concurrency Issues:**
   - Never reached concurrent execution phase
   - Sibling project isolation testing would happen at runtime, but test never gets there

**What This Test Would Validate (If It Compiled):**
- Sibling projects continue serving events during one project's degradation
- API endpoints remain responsive for healthy projects
- Metrics collection continues uninterrupted
- No cascading failures across projects

**Hypothetical Flakiness Vectors (If Test Compiled):**
- Race conditions between daemon state updates and HTTP assertions
- Timeouts in detecting sibling project health
- Concurrent access to shared metrics structures
- Filesystem watcher latency in detecting `.beads/` changes

**BUT: These are hypothetical only - test never reaches execution.**

---

### Test 3: `test_readyz_response_format`

#### Classification: ✅ DETERMINISTIC FAILURE

**Confidence:** 100%

**Failure Point:** Compilation Phase - Test never reached execution

**Rationale:**

1. **Consistency of Failure:** 100% reproducible
   - Same compilation blocker as Tests 1 and 2
   - Part of the same test target, blocked by same errors

2. **No Timing Dependence:**
   - Fails during static compilation analysis
   - No runtime timeouts or delays

3. **No Resource Dependence:**
   - Not dependent on runtime state
   - Only dependent on code correctness

4. **No Concurrency Issues:**
   - Single-threaded compilation failure
   - No concurrent execution

**What This Test Would Validate (If It Compiled):**
- `/readyz` endpoint returns correct JSON schema
- HTTP 200 OK status when daemon is healthy
- `status` field set to `"ok"`
- `degraded_projects` list is empty when healthy

**Hypothetical Flakiness Vectors (If Test Compiled):**
- HTTP response timing variations
- JSON parsing edge cases
- Daemon startup race conditions (is daemon ready when test probes?)

**BUT: These are hypothetical only - test never reaches execution.**

---

## Cross-Cutting Analysis: Why All Failures Are Deterministic

### The "Compilation Gate" Pattern

All three tests are blocked by the **same deterministic compilation failure**. This is a **"target-wide compilation guard"** pattern:

1. Rust compiles the entire `hoop-daemon` test target as one unit
2. Compilation errors in ANY file in the target block the ENTIRE target
3. Without a compiled test binary, NO tests can execute
4. The blocker is in unrelated test files (`property_invariants.rs`, `draft_queue_invariants.rs`)

**Deterministic Nature:**
- Compiler errors are all-or-nothing: code either compiles or it doesn't
- No randomness, no timing, no environmental factors
- Same source code → same compilation result → 100% reproducible

### Why Flaky Tests Require Execution

Flaky behavior can only emerge during **test execution**, not compilation:

| Phase | Can Be Flaky? | Reason |
|-------|--------------|--------|
| **Compilation** | ❌ No | Deterministic static analysis; no runtime state |
| **Test Execution** | ✅ Yes | Runtime state, timing, concurrency, resources |

**These tests never reach execution, so they cannot exhibit flakiness.**

### What Flakiness Would Look Like (If Tests Compiled)

If these tests compiled and ran, potential flakiness could arise from:

**Timing-Dependent Flakiness:**
- Daemon startup detection (polling vs. ready)
- Filesystem watcher latency in detecting `.beads/` deletion
- HTTP request timeout settings
- State propagation delays between components

**Concurrency-Dependent Flakiness:**
- Race conditions between HTTP requests and internal state updates
- Concurrent access to shared project state structures
- Lock contention in metrics collection
- Non-deterministic ordering of concurrent events

**Resource-Dependent Flakiness:**
- Port binding conflicts (ephemeral port exhaustion)
- Temporary directory cleanup failures
- File descriptor limits
- Memory pressure during daemon spawn

**BUT: None of these apply because tests never compile.**

---

## Flaky Test Detection Heuristics

For future reference, here are heuristics to identify flaky tests:

### Indicators of Flaky Tests

1. **Intermittent Passing:**
   - Test passes sometimes, fails other times, with no code changes
   - CI builds show non-deterministic pass/fail patterns

2. **Timing-Related Failures:**
   - Timeouts in otherwise fast operations
   - "element not found" after explicit waits
   - Race conditions in logs

3. **Concurrency-Related Failures:**
   - Different results when run in parallel vs. serial
   - Lock acquisition failures
   - Deadlocks or livelocks

4. **Resource-Related Failures:**
   - Port binding errors (EADDRINUSE)
   - File descriptor exhaustion
   - Out-of-memory errors
   - Disk space exhaustion

5. **Environment-Related Failures:**
   - Differences across OS platforms
   - Differences across Rust versions
   - Differences across dependency versions

### Indicators of Deterministic Tests

1. **Consistent Failures:**
   - Same error every time, same stack trace
   - Failure is invariant across runs
   - Error is logical/semantic, not environmental

2. **Compilation/Type Errors:**
   - Static analysis rejects code
   - Type mismatches, missing imports, syntax errors
   - No execution required to observe failure

3. **Logical Errors:**
   - Test asserts something that's factually wrong
   - Mocks/stubs don't match production behavior
   - Test data doesn't match schema

**These `beads_deletion_http` tests fall squarely in the "Deterministic" category.**

---

## Recommendations

### Immediate: Fix Compilation Blockers

**Priority 1: Fix `property_invariants.rs` (19 errors)**

```rust
// Add missing imports
use std::fs::File;
use std::io::BufRead;

// Fix proptest strategy capture (6 occurrences)
// Convert from:
fn strategy() { prop::collection::vec(event_strategy, 0..20) }
// To closure form:
fn strategy() || { prop::collection::vec(event_strategy, 0..20) }

// Fix StitchStatus Hash trait bound
// Either add #[derive(Hash)] to StitchStatus, or:
let status = ctx.derive_status();
results.insert(status.clone());  // Clone instead of move

// Fix use-after-move
prop_assert_eq!(status1, status2.clone(), "First and second calls differ");
prop_assert_eq!(status2, status3, "Second and third calls differ");
```

**Priority 2: Fix `draft_queue_invariants.rs` (2 errors)**

```rust
// Update DraftRow initializers with all required fields
let draft = hoop_daemon::fleet::DraftRow {
    id: "test-draft".to_string(),
    project_id: "test-project".to_string(),
    title: "Test Draft".to_string(),
    description: "Test description".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
    abandoned_at: None,                    // ← Missing field
    last_autosave_at: None,              // ← Missing field
    opened_at: None,                      // ← Missing field
    // ... and 2 other missing fields
};
```

### Follow-Up: Re-classify After Compilation Fixes

**Once compilation is fixed, re-run tests and re-classify:**

```bash
# Fix compilation errors first
cargo check --tests

# Then run the tests
cargo test -p hoop-daemon --test beads_deletion_http

# Analyze runtime failures for flakiness indicators:
# - Intermittent passing?
# - Timeouts?
# - Race conditions?
# - Port conflicts?
```

**New Classification Task (Future Bead):**
- After compilation fixes, if tests fail at runtime, classify those failures
- Look for timing-dependent behavior, concurrency issues, resource dependence
- Document whether runtime failures are deterministic or flaky

### Long-term: Prevent Flaky Tests

**Test Design Principles to Avoid Flakiness:**

1. **Avoid Time-Based Assertions:**
   - ❌ `assert_eq!(time.elapsed().as_secs(), 1)`
   - ✅ `assert!(time.elapsed().as_secs() < 5)`

2. **Use Explicit Synchronization:**
   - ❌ `sleep(100ms); assert_ready();`
   - ✅ `wait_for_ready(timeout).await;`

3. **Isolate Resources:**
   - ❌ `bind_port(8080)`  // Fixed port, may conflict
   - ✅ `bind_ephemeral_port()`  // Random port, no conflicts

4. **Make Tests Order-Independent:**
   - ❌ Tests share global state
   - ✅ Each test owns its state

5. **Retry Transient Failures:**
   - Use `retry` library for network/disk operations
   - Exponential backoff for retries

---

## Conclusion

### Summary of Findings

**All three `beads_deletion_http` tests are DETERMINISTIC FAILURES.**

| Test | Classification | Confidence | Reason |
|------|---------------|------------|--------|
| `test_beads_deletion_readyz_degraded` | Deterministic | 100% | Compilation block, no execution |
| `test_beads_deletion_sibling_events_continue` | Deterministic | 100% | Compilation block, no execution |
| `test_readyz_response_format` | Deterministic | 100% | Compilation block, no execution |

**Key Insight:** Flakiness requires test execution. These tests never execute, so they cannot be flaky. The compilation errors are deterministic: same source code produces same compilation errors 100% of the time.

### Next Steps

1. **Fix compilation errors** in `property_invariants.rs` (19 errors) and `draft_queue_invariants.rs` (2 errors)
2. **Re-run tests** to assess runtime behavior
3. **Re-classify** if runtime failures occur (look for timing, concurrency, resource dependencies)
4. **Design flakiness-resistant tests** when writing new tests

### Flaky Test Suspected Causes (For Future Runtime Failures)

If tests fail at runtime after compilation fixes, investigate:

**Timing-Dependent:**
- Daemon startup polling timeouts
- Filesystem event propagation delays
- HTTP response latency variations

**Concurrency-Dependent:**
- Race conditions in state updates
- Lock contention in shared structures
- Non-deterministic event ordering

**Resource-Dependent:**
- Port binding conflicts (use ephemeral ports)
- Temporary directory cleanup (use proper scoped tempdirs)
- File descriptor limits (ensure cleanup)

**BUT: These are speculative. Tests must compile and execute first to exhibit such behavior.**

---

**Severity:** HIGH - Tests cannot verify ANY functionality until compilation is fixed

**Test Readiness:** Tests are well-structured and comprehensive, but completely blocked by deterministic compilation errors

**Functional Coverage:** Once compilation is fixed, these tests will validate health monitoring, multi-tenant isolation, and API contracts - all critical for Phase 1 exit gate

---

## References

- **Source bead:** bf-4l8jp (raw test output extraction)
- **Failure documentation:** bf-wss0q (detailed failure points and error details)
- **Initial classification:** bf-236t4 (test classification and functionality context)
- **Plan reference:** §6 Phase 2 success criteria, §3.9 (readiness probe requirements)
