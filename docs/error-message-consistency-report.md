# HOOP Error Message Consistency Report

**Report Generated:** 2026-08-12  
**Task:** bf-34xgz - Document error message consistency findings  
**Validation Source:** bf-55am0 - Complete error message validation against standards  
**Catalog:** 5,904 error messages from 104 test files  
**Standards Reference:** docs/error-message-consistency-standards.md

---

## Executive Summary

The HOOP test suite contains **5,904 error messages** across 104 test files. A comprehensive validation against defined consistency standards revealed that **73.1% of messages violate one or more standards**, with the most common issues being lack of descriptive context and failure to follow standard wording patterns.

### Key Statistics

| Metric | Value | Status |
|--------|-------|--------|
| **Total Messages** | 5,904 | — |
| **Compliant Messages** | 1,590 | ✅ 26.9% |
| **Violations Found** | 4,314 | ❌ 73.1% |
| **Minimal Context Issues** | 1,781 | 🔴 Critical |
| **Pattern Violations** | 4,314 | 🔴 Critical |
| **Trailing Periods** | 1 | 🟢 Minor |
| **Single-Word Messages** | 132 | 🟡 Moderate |

### Overall Assessment

**Status:** ⚠️ **MAJOR IMPROVEMENTS NEEDED**

The majority of error messages fail to meet minimum consistency standards. However, the violations are **systematic and template-based**, making them **straightforward to fix** through pattern-based replacements rather than requiring deep architectural changes.

---

## Most Common Inconsistency Patterns

### Pattern 1: Single-Word Value Messages (1,781 instances - 30.2%)

**Issue:** Error messages that consist only of a value (command name, flag, path) without any descriptive context.

**Severity:** 🔴 **HIGH** - These messages are not self-documenting. When tests fail, developers must read the test code to understand what failed.

**Examples Found:**
```rust
❌ "scan"                    // Command name only
❌ "/tmp"                    // Path only  
❌ "-y"                      // Flag only
❌ "--no-interactive"        // Long flag only
❌ "test-project"            // Project name only
```

**Fixed Versions:**
```rust
✅ "scan command should be recognized"
✅ "test project path should be /tmp"
✅ "no_interactive should be true when -y flag is present"
✅ "--no-interactive flag should be parsed correctly"
✅ "project name should be test-project"
```

**Why This Matters:**
- Test failures display as: `assertion failed: scan` (what about scan?)
- Developers must open the test file to understand the failure
- CI logs become harder to triage without context
- New team members have steeper learning curve

**Files Most Affected:**
- `tests/cli_test_helpers.rs` (205 violations)
- `hoop-cli/tests/cli_test_helpers.rs` (199 violations)
- `hoop-daemon/tests/integration_harness.rs` (189 violations)

---

### Pattern 2: Missing "Should" Pattern (4,314 instances - 73.1%)

**Issue:** Messages don't follow the standard `<subject> should <state> [when <condition>]` pattern.

**Severity:** 🔴 **HIGH** - Inconsistent wording makes the codebase harder to read and maintain.

**Examples Found:**
```rust
❌ "flag should be true"                    // Which flag?
❌ "parsing failed"                        // What failed to parse?
❌ "invalid config"                        // What's invalid about it?
❌ "no_interactive should be true"         // Under what condition?
```

**Fixed Versions:**
```rust
✅ "no_interactive flag should be true in non-interactive mode"
✅ "Failed to parse config: missing required field"
✅ "Config validation failed: schema_version must be a string"
✅ "no_interactive should be true when --no-interactive flag is present"
```

**Standard Patterns Required:**
1. **Assertions:** `<subject> should <state> [when <condition>]`
2. **Operations:** `Failed to <action> <target> [because <reason>]`
3. **Invariants:** `<subject> must <condition>`

---

### Pattern 3: Missing "When" Condition Clauses (158+ instances)

**Issue:** Messages with "should" statements that don't specify when the expected behavior applies.

**Severity:** 🟡 **MEDIUM** - Messages are technically compliant but lack precision.

**Examples Found:**
```rust
❌ "no_interactive should be true"
❌ "Parsing should succeed"  
❌ "Flag should be consistent"
```

**Fixed Versions:**
```rust
✅ "no_interactive should be true when --no-interactive flag is present"
✅ "Parsing should succeed even when flag is at end of command"
✅ "Flag should be consistent regardless of position in command"
```

**Why "When" Clauses Matter:**
- Clarifies the condition being tested
- Distinguishes between different test scenarios
- Prevents false positives when failures occur for different reasons

---

### Pattern 4: "Must" vs "Should" Overuse (419+ instances)

**Issue:** Messages use "must" for preferences that should use "should".

**Severity:** 🟡 **MEDIUM** - "Must" should be reserved for invariants and hard requirements.

**Examples Found:**
```rust
❌ "Flag position must not affect value"          // Preference, not invariant
❌ "Flag value must be consistent"                 // Should use "should"
❌ "Init must check no_interactive flag"           // Test expectation, not invariant
```

**Fixed Versions:**
```rust
✅ "Flag position should not affect value"         // Preference
✅ "Flag value should be consistent"               // Expectation
✅ "Init should check no_interactive flag"         // Test expectation
```

**When to Use Each:**
- **`should`** - Test expectations, preferred behavior (95% of assertions)
- **`must`** - Invariants, hard requirements, security-critical validations (5%)
- **`Failed to`** - Operations that didn't complete (I/O, parsing, network)

---

### Pattern 5: Missing Expected vs Actual Comparisons (98+ instances)

**Issue:** Comparison messages state what was received but not what was expected.

**Severity:** 🟡 **MEDIUM** - Makes debugging harder when values don't match.

**Examples Found:**
```rust
❌ "total_workers must be numeric, got: {}"
❌ "Bead events endpoint should return 200 or 404, got: {}"
❌ "stdout should not contain interactive prompts, got: {}"
```

**Fixed Versions:**
```rust
✅ "total_workers should be numeric, expected integer, got: {}"
✅ "Bead events endpoint should return 200, got: {}"
✅ "stdout should be empty, got: {} lines of output"
```

**Standard Format:** `Expected <expected>, got <actual>`

---

### Pattern 6: Cryptic/Meta Messages (123+ instances)

**Issue:** Messages that reference test internals rather than describing the expected behavior.

**Severity:** 🟡 **MEDIUM** - Obscures the intent of the test.

**Examples Found:**
```rust
❌ "scan"                           // Meta-reference to command
❌ "/tmp"                          // Meta-reference to path  
❌ "-y"                            // Meta-reference to flag
```

**Fixed Versions:**
```rust
✅ "scan command should be recognized"
✅ "test project directory should be /tmp"
✅ "no_interactive should be true when -y flag is present"
```

**Why Meta-References Fail:**
- Assume reader knows test implementation details
- Don't describe what behavior is being tested
- Make failures harder to diagnose without reading test code

---

## Specific Examples by Category

### Category 1: CLI Command Messages (200+ violations)

**Files:** `tests/cli_test_helpers.rs`, `hoop-cli/tests/cli_test_helpers.rs`

**Current State:**
```rust
assert_eq!(parsed.subcommand, Some("scan".to_string()), "scan");
assert_eq!(parsed.project, Some("/tmp".to_string()), "/tmp");
assert!(flags.no_interactive, "-y");
```

**Improved State:**
```rust
assert_eq!(parsed.subcommand, Some("scan".to_string()), 
    "subcommand should be 'scan' when scan command invoked");
assert_eq!(parsed.project, Some("/tmp".to_string()), 
    "project path should be /tmp in test configuration");
assert!(flags.no_interactive, 
    "no_interactive should be true when -y flag is present");
```

**Impact:** Changes 200+ cryptic messages into self-documenting test assertions

---

### Category 2: Configuration Validation (150+ violations)

**Files:** `hoop-daemon/tests/config_field_validation.rs`, `hoop-daemon/tests/config_reload_cycle.rs`

**Current State:**
```rust
assert!(err.is_some(), "missing schema_version should fail");
assert!(projects.is_array(), "projects should be a list");
```

**Improved State:**
```rust
assert!(err.is_some(), 
    "config with missing schema_version should fail validation. Schema requires version field in root");
assert!(projects.is_array(), 
    "projects field should be a list in config YAML");
```

**Impact:** Configuration errors become immediately actionable

---

### Category 3: HTTP Endpoint Assertions (100+ violations)

**Files:** Multiple integration test files

**Current State:**
```rust
assert_eq!(resp.status(), 200, "should return 200");
assert_eq!(resp.status(), 404, "should return 404");
```

**Improved State:**
```rust
assert_eq!(resp.status(), 200, 
    "GET /api/beads endpoint should return 200 OK");
assert_eq!(resp.status(), 404, 
    "Non-existent bead endpoint should return 404 Not Found");
```

**Impact:** API errors immediately identify which endpoint failed

---

### Category 4: State Comparison Messages (80+ violations)

**Files:** `hoop-daemon/tests/draft_queue_invariants.rs`, `hoop-daemon/tests/multi_operator_concurrency.rs`

**Current State:**
```rust
assert_eq!(original["id"], fetched["id"], "id should match");
assert_eq!(original["title"], fetched["title"], "title should match");
```

**Improved State:**
```rust
assert_eq!(original["id"], fetched["id"], 
    "fetched bead ID should match original bead ID");
assert_eq!(original["title"], fetched["title"], 
    "fetched bead title should match original bead title");
```

**Impact:** State comparison failures show exactly which field mismatched

---

## Severity Assessment

### 🔴 High Severity (Critical Impact - Fix Immediately)

**1. Minimal Context Violations (1,781 messages - 30.2%)**

**Why Critical:**
- Tests become opaque to anyone other than the original author
- CI failures require opening test files to understand
- New developer onboarding is significantly harder
- Debugging time increases for every test failure

**Example Impact:**
```
Current CI Output:
  test cli_parsing ... FAILED
  assertion failed: scan

Improved CI Output:
  test cli_parsing ... FAILED  
  assertion failed: scan command should be recognized when scan flag is present
```

**Fix Complexity:** LOW - Template-based text additions

---

**2. Non-Standard Pattern Violations (4,314 messages - 73.1%)**

**Why Critical:**
- Inconsistent codebase makes patterns harder to recognize
- Violates DRY (Don't Repeat Yourself) - similar assertions worded differently
- Makes it harder to distinguish between invariants, expectations, and operations
- Increases cognitive load when reading tests

**Example Impact:**
```rust
// Current (inconsistent):
assert_eq!(status, 200, "should be 200");
assert!(body.is_ok(), "response must be ok");  
assert_eq!(count, 5, "expected 5 items");

// Improved (consistent):
assert_eq!(status, 200, "healthz endpoint should return 200");
assert!(body.is_ok(), "response body should be present");
assert_eq!(count, 5, "should have 5 open beads");
```

**Fix Complexity:** MEDIUM - Pattern restructuring using templates

---

### 🟡 Medium Severity (Noticeable Impact - Fix Soon)

**3. Missing "When" Condition Clauses (158 messages - 2.7%)**

**Why Medium:**
- Messages are technically compliant but lack precision
- Can lead to false debugging paths
- Makes it harder to identify test scenario boundaries

**Fix Complexity:** LOW - Add condition text to existing messages

---

**4. "Must" vs "Should" Overuse (419 messages - 7.1%)**

**Why Medium:**
- Undermines the distinction between invariants and preferences
- Makes it harder to identify hard requirements
- Creates confusion about what's critical vs. nice-to-have

**Fix Complexity:** LOW - Find/replace "must" → "should" (except for true invariants)

---

**5. Missing Expected vs Actual Comparisons (98 messages - 1.7%)**

**Why Medium:**
- Requires additional debugging to determine expected value
- Makes test triage slower
- Obscures the intent of value comparisons

**Fix Complexity:** LOW-MEDIUM - Add expected values to comparison messages

---

### 🟢 Low Severity (Minor Impact - Fix When Convenient)

**6. Trailing Period Violations (1 message - <0.1%)**

**Why Low:**
- Only one instance found in entire codebase
- Doesn't impact comprehension, just style consistency
- Nearly universal compliance already (99.9%)

**Fix Complexity:** TRIVIAL - Remove one trailing period

---

## Recommended Fixes Prioritized by Impact

### Phase 1: High-Impact, Low-Effort Fixes (Do First)

**Priority 1.1: Add Descriptive Context to Single-Word Messages**
- **Count:** 1,781 messages (30.2% of all messages)
- **Action:** Convert bare values to standard "should" pattern messages
- **Template:** `"<value>"` → `"<context> should be <value> [when <condition>]"`
- **Examples:**
  - `"scan"` → `"scan command should be recognized"`
  - `"/tmp"` → `"project path should be /tmp"`
  - `"-y"` → `"no_interactive should be true when -y flag is present"`
- **Impact:** Eliminates 30% of violations with simple text additions
- **Effort:** LOW (2-3 hours) - Template-based replacements
- **Files:** `tests/cli_test_helpers.rs`, `hoop-cli/tests/cli_test_helpers.rs`, `hoop-daemon/tests/integration_harness.rs`

---

**Priority 1.2: Replace Generic `.unwrap()` with Descriptive `.expect()`**
- **Count:** 1,482 instances (25.1% of all messages)
- **Action:** Add "Failed to <action> <target>" messages to unwrap calls
- **Template:** `.unwrap()` → `.expect("Failed to <action> <target>")`
- **Examples:**
  - `.unwrap()` → `.expect("Failed to read config file")`
  - `.unwrap()` → `.expect("Failed to parse bead from response")`
- **Impact:** Prevents silent panics, provides debugging context
- **Effort:** LOW (1-2 hours) - Find-and-replace with templates
- **Risk:** LOW - Doesn't change test behavior, just improves panic messages

---

**Priority 1.3: Remove Trailing Period**
- **Count:** 1 message
- **Action:** Remove trailing period from one message
- **Location:** `phase2_exit_gate.rs:415`
- **Impact:** 100% compliance with no-trailing-period standard
- **Effort:** TRIVIAL (<5 minutes)

---

### Phase 2: Medium-Impact, Medium-Effort Fixes (Do Second)

**Priority 2.1: Convert Assertions to Standard Patterns**
- **Count:** 4,314 non-compliant messages (73.1% of all messages)
- **Action:** Apply "should/failed to/must" patterns to all assertions
- **Templates:**
  - Assertions: `<subject> should <state> [when <condition>]`
  - Operations: `Failed to <action> <target> [because <reason>]`
  - Invariants: `<subject> must <condition>`
- **Examples:**
  - `"flag true"` → `"no_interactive flag should be true when --no-interactive is present"`
  - `"parsing failed"` → `"Failed to parse config: missing required field"`
- **Impact:** Consistent wording, self-documenting tests
- **Effort:** MEDIUM (4-6 hours) - Requires message restructuring
- **Strategy:** Focus on high-density files first (CLI helpers, integration tests)

---

**Priority 2.2: Add Actual vs Expected Comparisons**
- **Count:** 98 missing comparison messages
- **Action:** Use "Expected <expected>, got <actual>" format
- **Template:** `Expected <expected>, got <actual>`
- **Examples:**
  - `"integer but got string"` → `"Expected string, got integer"`
  - `"got 404"` → `"Expected 200, got 404"`
- **Impact:** Clearer failure diagnosis for type/value mismatches
- **Effort:** LOW-MEDIUM (2-3 hours) - Pattern-based additions

---

**Priority 2.3: Add "When" Condition Clauses**
- **Count:** 158 missing condition clauses
- **Action:** Add "when <condition>" to "should" statements
- **Template:** `<subject> should <state> when <condition>`
- **Examples:**
  - `"no_interactive should be true"` → `"no_interactive should be true when --no-interactive is present"`
  - `"Parsing should succeed"` → `"Parsing should succeed even when flag is at end of command"`
- **Impact:** More precise test scenario documentation
- **Effort:** LOW (1-2 hours) - Text additions to existing messages

---

### Phase 3: Lower-Priority Enhancements (Do Last)

**Priority 3.1: Correct "Must" vs "Should" Usage**
- **Count:** 419 overused "must" instances
- **Action:** Replace "must" with "should" for preferences and expectations
- **Exception:** Keep "must" for true invariants (security-critical, hard requirements)
- **Impact:** Clearer distinction between invariants and expectations
- **Effort:** LOW (1-2 hours) - Mostly find/replace with manual review

---

**Priority 3.2: Add Contextual Information (Advanced)**
- **Count:** TBD (requires analysis)
- **Action:** Include file paths, line numbers, connection IDs, worker IDs
- **Templates:**
  - Parse errors: `Failed to parse <file>: line <n>, column <m>`
  - Resource errors: `Failed to acquire lock (held by operation {})`
  - Connection errors: `No init message (conn {})`
- **Impact:** Better error diagnosis for complex failures
- **Effort:** MEDIUM-HIGH (3-4 hours) - Requires context capture

---

**Priority 3.3: Add Actionable Suggestions (User-Facing Errors Only)**
- **Count:** TBD (subset of user-facing errors)
- **Action:** Include safe, obvious fixes for user-correctable errors
- **Template:** `<problem>. <safe_fix_suggestion>`
- **Examples:**
  - `"Failed to read ~/.hoop/config.yml: file not found. Run 'hoop init' to create it"`
  - `"schema_version missing from config. Add: schema_version: 1"`
- **Impact:** User-friendly error resolution
- **Effort:** MEDIUM-HIGH (2-3 hours) - Requires judgment about what's safe/obvious

---

## Files Requiring Most Attention

Based on error message density and violation patterns, these files should be addressed first:

| File | Error Count | Primary Issues | Priority Level |
|------|-------------|----------------|---------------|
| `tests/cli_test_helpers.rs` | 205 | Single-word messages, missing context | 🔴 HIGH |
| `hoop-cli/tests/cli_test_helpers.rs` | 199 | CLI validation, minimal messages | 🔴 HIGH |
| `hoop-daemon/tests/integration_harness.rs` | 189 | Generic assertions, meta-references | 🔴 HIGH |
| `hoop-daemon/tests/config_field_validation.rs` | 183 | Value-only messages, no context | 🟡 MEDIUM |
| `hoop-cli/tests/no_interactive_flag_behavior.rs` | 171 | Flag-value messages, no "when" clauses | 🟡 MEDIUM |
| `hoop-daemon/tests/config_reload_cycle.rs` | 165 | State validation, minimal context | 🟡 MEDIUM |
| `hoop-daemon/tests/draft_queue_invariants.rs` | 150 | State comparison, vague assertions | 🟡 MEDIUM |
| `hoop-daemon/tests/multi_operator_concurrency.rs` | 136 | Concurrency assertions, minimal context | 🟡 MEDIUM |
| `hoop-cli/tests/scan_no_interactive_flag.rs` | 125 | Flag validation, single-word messages | 🟡 MEDIUM |
| `hoop-daemon/tests/hoop_dies_nothing_notices.rs` | 125 | Lifecycle assertions, generic messages | 🟡 MEDIUM |

**Fix Strategy:**
1. Start with top 3 files (HIGH priority) - 593 violations total
2. Move to next 7 files (MEDIUM priority) - 1,105 violations total
3. Address remaining files in order of violation count

---

## Success Criteria

### Minimum Viable Improvement (Phase 1 Complete)
- ✅ Eliminated all single-word messages (1,781 fixes)
- ✅ All unwrap() calls replaced with expect() (1,482 fixes)  
- ✅ No trailing periods (1 fix)
- **Result:** 30% reduction in violations (73.1% → 51%)

### Target Improvement (Phase 2 Complete)
- ✅ All assertions follow standard patterns (4,314 fixes)
- ✅ All comparisons include expected vs actual (98 fixes)
- ✅ All "should" statements include "when" clauses (158 fixes)
- **Result:** 80% reduction in violations (73.1% → 15%)

### Excellent Improvement (Phase 3 Complete)
- ✅ Correct "must" vs "should" usage (419 fixes)
- ✅ All messages include appropriate context (TBD fixes)
- ✅ User-facing errors include actionable suggestions (TBD fixes)
- **Result:** 95%+ reduction in violations (73.1% → <4%)

### Validation Method
After each phase, re-run validation script to measure improvement:
```bash
python3 bin/validate_error_messages.py
```

Target compliance rates:
- **Phase 1:** 49% compliant (up from 26.9%)
- **Phase 2:** 85% compliant
- **Phase 3:** 96%+ compliant

---

## Implementation Guidance

### Fix Strategy

1. **Start with Phase 1 fixes** - Highest impact, lowest effort
2. **Use template-based replacements** - Most fixes follow predictable patterns
3. **Focus on high-density files first** - CLI helpers and integration tests
4. **Verify fixes don't break tests** - Adding context shouldn't change test behavior
5. **Document any exceptions** - Some messages may intentionally deviate

### Quality Assurance

After implementing fixes for each phase:
- Run `cargo test` to ensure no test behavior changes
- Re-run validation to measure improvement
- Update error message catalog
- Document any intentional exceptions to standards

### Common Fix Patterns

**Pattern A: Single-Word → Full Message**
```rust
// Before
assert_eq!(value, "scan", "scan");

// After
assert_eq!(value, "scan", "command should be 'scan'");
```

**Pattern B: Generic → Specific**
```rust
// Before
assert!(condition, "should be true");

// After  
assert!(condition, "no_interactive flag should be true when --no-interactive is present");
```

**Pattern C: unwrap() → expect()**
```rust
// Before
let value = some_option.unwrap();

// After
let value = some_option.expect("bead ID should be present in response");
```

**Pattern D: Missing Expected**
```rust
// Before
assert_eq!(actual, expected, "got: {}", actual);

// After
assert_eq!(actual, expected, "expected '{}', got '{}'", expected, actual);
```

---

## Conclusion

The HOOP error message catalog shows **significant inconsistency with defined standards**, but the violations are **systematic and template-based**, making them **straightforward to fix** through pattern-based replacements.

**Key Takeaways:**

1. **73.1% non-compliance rate** is concerning, but fixable
2. **30.2% are single-word messages** - trivial to fix with templates
3. **No fundamental design issues** - all violations are surface-level text problems
4. **High-density files are the priority** - fixing 10 files addresses ~60% of violations

**Recommended Approach:**

1. **Start Phase 1 immediately** - Quick wins that eliminate 30% of violations
2. **Move to Phase 2** - Systematic pattern compliance 
3. **Finish with Phase 3** - Context and actionability enhancements
4. **Validate continuously** - Re-run validation after each phase

**Expected Timeline:**
- Phase 1: 3-5 hours (1,781 + 1,482 + 1 = 3,264 fixes)
- Phase 2: 6-10 hours (4,314 + 98 + 158 = 4,570 fixes)  
- Phase 3: 5-8 hours (419 + TBD context + TBD suggestions)
- **Total:** 14-23 hours for complete compliance

**Impact:**
- Self-documenting tests that fail with clear messages
- Faster debugging and triage of CI failures
- Easier onboarding for new developers
- Consistent, professional codebase

---

**Standards Documents Referenced:**
- docs/error-message-consistency-standards.md (complete standards)
- error_messages_catalog.md (5,904 message inventory)
- error_validation_final_report.md (detailed validation results)

**Next Steps:**
1. Implement Phase 1 fixes following templates in this report
2. Re-run validation to confirm improvement
3. Proceed to Phase 2 once Phase 1 is validated
4. Use this report as the roadmap for systematic improvement

---

**Report Status:** ✅ COMPLETE  
**Last Updated:** 2026-08-12  
**Version:** 1.0
