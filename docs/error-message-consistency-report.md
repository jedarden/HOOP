# HOOP Error Message Consistency Report

**Generated:** 2026-08-12  
**Bead:** bf-34xgz  
**Based on:** Validation report (bf-55am0) and consistency standards (bf-4vtp7)  
**Total Messages Analyzed:** 5,904 error messages across 104 test files

---

## Executive Summary

HOOP's error messages have **significant consistency issues** that impact debugging efficiency and code maintainability. Overall compliance with defined standards is **33.8%**, with **3,910 violations** across four categories.

**Key Findings:**
- **Wording issues** are the most prevalent problem (45.3% of all violations)
- **Actionability** is the second most common issue (11.0% of violations)
- **Context information** is frequently missing (9.9% of violations)
- **Formatting** is done correctly (0% violations)

**Good News:** The standards are well-defined, examples exist, and most fixes are straightforward automated improvements.

---

## Problem Impact

### Why This Matters

**1. Debugging Efficiency**
Inconsistent error messages waste developer time. When tests fail, developers must read the code to understand what went wrong instead of getting clear, actionable feedback from the error message itself.

**Example:**
```rust
// Current - wastes time
assert!(result.is_err(), "test-project");

// Improved - self-documenting
assert!(result.is_err(), "project name should be 'test-project'");
```

**2. Onboarding and Maintenance**
New developers joining the HOOP project need to learn patterns from well-structured examples. Inconsistent messages create cognitive overhead and make the codebase harder to navigate.

**3. Test Reliability**
Vague error messages lead to misinterpreted test failures. When a test says "arg1" failed, developers don't know if it's a parsing error, validation error, or something else without reading the test code.

---

## Violation Breakdown

### Summary Statistics

| Category | Violations | Percentage | Severity | Fix Complexity |
|----------|-----------|------------|----------|----------------|
| **Wording Issues** | 2,677 | 45.3% | High | Low (automatable) |
| **Not Actionable** | 648 | 11.0% | Medium | Medium |
| **Missing Information** | 585 | 9.9% | Medium | Medium |
| **Formatting Issues** | 0 | 0.0% | Low | N/A (already correct) |
| **Good Examples** | 949 | 16.1% | N/A | N/A (reference) |

**Total Violations:** 3,910  
**Good Examples:** 949 (16.1%)  
**Compliance Rate:** 33.8%

---

## Most Common Issues (Priority Order)

### 1. Incorrect Case (1,885 instances - 48.2% of violations)

**Problem:** Messages don't follow sentence case conventions.

**Standard:** First word capitalized, rest lowercase (except proper nouns like HOOP, HTTP).

**Examples:**
```rust
❌ "no_interactive should be true with flag before command"
✅ "No_interactive should be true with flag before command"

❌ "projects should be a list"
✅ "Projects should be a list"
```

**Impact:** High - Makes error messages harder to scan and appear unprofessional.

**Fix:** Capitalize first letter of all messages. Can be automated with simple regex.

---

### 2. Missing "Should" Pattern (791 instances - 20.2% of violations)

**Problem:** Assertion and validation messages don't use the standard "should {verb}" pattern.

**Standard:** Use descriptive "should {verb}" language instead of bare values.

**Examples:**
```rust
❌ "scan"
❌ "test-project"
❌ "arg1"

✅ "Command should be 'scan'"
✅ "Project name should be 'test-project'"
✅ "First argument should be 'scan'"
```

**Impact:** High - Missing "should" pattern makes messages less descriptive and harder to understand.

**Fix:** Add descriptive "should {verb}" language to assertions. Partially automatable.

---

### 3. Not Actionable (648 instances - 16.6% of violations)

**Problem:** Messages are vague and don't suggest what went wrong or what's expected.

**Standard:** Include context about what's being validated and describe the expected state.

**Examples:**
```rust
❌ "test-project"
❌ "No arguments provided"
❌ "new-project"

✅ "Project name should be 'test-project'"
✅ "Command should require at least one argument"
✅ "Project name should be 'new-project'"
```

**Impact:** Medium - Vague messages waste developer time during debugging.

**Fix:** Add context about what's being validated and include relevant identifiers.

---

### 4. Missing Information (585 instances - 15.0% of violations)

**Problem:** Messages lack sufficient context for debugging (identifiers, field paths, endpoints).

**Standard:** Include identifiers, field paths, HTTP endpoints where relevant.

**Examples:**
```rust
❌ "arg1"
❌ "Some tests failed: {:?}"

✅ "First argument should be 'scan', got 'status'"
✅ "Tests in suite '{}' should all pass: {:?}"
```

**Impact:** Medium - Developers can't identify the source of failures without additional debugging.

**Fix:** Include bead IDs, project names, field paths, HTTP endpoints in messages.

---

## Files Requiring Most Attention

### Top 10 Files by Violation Count

| Rank | File | Violations | Primary Issues |
|------|------|------------|----------------|
| 1 | `hoop-daemon/tests/draft_queue_invariants.rs` | 205 | incorrect_case (122), vague_message (41) |
| 2 | `tests/cli_test_helpers.rs` | 159 | missing_should_pattern (73), incorrect_case (58) |
| 3 | `hoop-daemon/tests/multi_operator_concurrency.rs` | 136 | incorrect_case (73), vague_message (53) |
| 4 | `hoop-daemon/tests/config_field_validation.rs` | 131 | incorrect_case (119), lacks_context (12) |
| 5 | `hoop-daemon/tests/hoop_dies_nothing_notices.rs` | 125 | incorrect_case (75), vague_message (48) |
| 6 | `hoop-daemon/tests/config_reload_cycle.rs` | 117 | incorrect_case (63), vague_message (24) |
| 7 | `hoop-cli/tests/cli_test_helpers.rs` | 109 | missing_should_pattern (42), lacks_context (34) |
| 8 | `hoop-daemon/tests/needle_events_roundtrip.rs` | 104 | incorrect_case (68), missing_should_pattern (33) |
| 9 | `hoop-daemon/tests/create_stitch_no_auto_submit.rs` | 101 | incorrect_case (53), vague_message (34) |
| 10 | `hoop-daemon/tests/adapter_failover.rs` | 98 | incorrect_case (46), vague_message (41) |

**Pattern:** Most violations are in daemon integration tests, which have complex assertion scenarios that need better context.

---

## Good Examples (Reference Patterns)

### Excellent Compliance Examples

These messages (949 total) follow all standards correctly and should be used as templates:

```rust
✅ "Should parse with flag before command"
✅ "Should parse with flag after command"
✅ "Should parse with -y flag"
✅ "Should parse without flag"
✅ "Flag should be true at any position"
✅ "Flag position should not affect value for {}"
```

**What Makes These Good:**
- Proper sentence case (first word capitalized)
- Include "should" pattern
- Descriptive and actionable
- Provide context about what's being tested
- No trailing periods

**Pattern:** Use these as templates when writing new test assertions.

---

## Recommended Fixes (Priority Order)

### Priority 1: High Impact, Quick Wins (2-3 hours)

**Target:** 2,677 violations (68.5% of all violations)

**1. Fix Sentence Case (1,885 instances)**
```bash
# Automated fix pattern
sed -i 's/"/"/' test_files  # Capitalize first letter of messages
```

**2. Add "Should" Pattern (791 instances)**
```rust
// Conversion examples
"scan" → "Command should be 'scan'"
"test-project" → "Project should be 'test-project'"
"--no-interactive" → "Flag should be '--no-interactive'"
```

**3. Remove Trailing Period (1 instance)**
```rust
// Quick fix
"Phase 2 exit gate FAILED..." → "Phase 2 exit gate FAILED.." (no period)
```

**Expected Outcome:** 
- Compliance rate: 33.8% → 75%+
- Remaining violations: ~1,200 (down from 3,910)

---

### Priority 2: Medium Impact (4-6 hours)

**Target:** 1,233 violations (31.5% of all violations)

**1. Add Context to Vague Messages (648 instances)**
```rust
// Before
"test-project"

// After
"Project name should be 'test-project'"
```

**2. Include Debugging Context (585 instances)**
```rust
// Before
"arg1"

// After
"First argument should be 'scan', got 'status'"
```

**Expected Outcome:**
- Compliance rate: 75% → 90%+
- Remaining violations: ~600 (down from ~1,200)

---

### Priority 3: Pattern Consolidation (2-3 hours)

**Target:** Remaining inconsistencies

**1. Consolidate Similar Patterns**
Identify messages with similar meaning but different wording, standardize on best-of-breed.

**2. Enhance Actionability**
Add "why" context for non-obvious assertions and fix hints where appropriate.

**Expected Outcome:**
- Compliance rate: 90% → 95%+
- Consistent patterns across codebase

---

## Fix Implementation Strategy

### Phase 1: Automated Fixes (Priority 1)

**Tools:** `sed`, `rg` (ripgrep), or custom Python script

**Process:**
1. Create backup of test files
2. Run automated fixes for sentence case
3. Run automated fixes for "should" pattern (where safe)
4. Remove trailing periods
5. Run validation script to verify improvements
6. Manual review of automated changes

**Validation:**
```bash
# Re-run validation after each batch
python3 scripts/validate_error_messages.py
```

---

### Phase 2: Manual Improvements (Priority 2)

**Approach:** File-by-file review of high-impact test files

**Priority Order:**
1. `hoop-daemon/tests/draft_queue_invariants.rs` (205 violations)
2. `tests/cli_test_helpers.rs` (159 violations)
3. `hoop-daemon/tests/multi_operator_concurrency.rs` (136 violations)

**Tasks:**
- Add context identifiers (IDs, names, paths)
- Improve vague messages with descriptive text
- Include HTTP endpoints and field paths

---

### Phase 3: Pattern Consolidation (Priority 3)

**Approach:** Identify and standardize similar patterns

**Tasks:**
1. Extract all messages, group by similarity
2. Vote on best phrasing for each pattern
3. Create message template library
4. Update code to use templates

---

## Quality Targets

### Current State
- **Compliance Rate:** 33.8%
- **Violations per 100 messages:** 66.2
- **Top Violation:** Incorrect case (48.2% of all violations)

### Target State (Post-Fix)
- **Compliance Rate:** 90%+
- **Violations per 100 messages:** <10
- **Top Violation:** None (all categories <5%)

### Success Criteria
- [ ] All Priority 1 fixes applied (sentence case, "should" pattern)
- [ ] Compliance rate reaches 75%+
- [ ] Top 10 files by violations have <20 violations each
- [ ] No single-word messages remain ("scan", "test-project", etc.)
- [ ] All validation errors include field paths or identifiers

---

## Reference Materials

### Standards Document
**Location:** `docs/error-message-consistency-standards.md` (bead bf-4vtp7)

**Contents:**
- Complete wording conventions
- Formatting patterns
- Context inclusion guidelines
- Actionability guidelines
- Error type standards
- Complete examples and anti-patterns

### Validation Report
**Location:** `error-message-validation-report.md` (bead bf-55am0)

**Contents:**
- Detailed violation examples per category
- File-by-file breakdown
- Sample violations and fix patterns
- Validation methodology

### Error Catalog
**Location:** `error_messages_catalog.md` (bead bf-3ysoc)

**Contents:**
- Complete inventory of 5,904 error messages
- Distribution by type and file
- Statistical breakdown

---

## Next Steps

### Immediate Actions
1. **Review this report** - Confirm the findings and approach
2. **Create fix bead** - Implement Priority 1 automated fixes
3. **Track progress** - Re-run validation after each fix batch

### Long-term Actions
1. **Integrate validation into CI** - Prevent regression of error message quality
2. **Create message templates** - Standard library of common error patterns
3. **Update contributor guidelines** - Add error message standards to onboarding docs

### Maintenance
1. **Quarterly validation** - Re-run validation to catch new inconsistencies
2. **Pre-commit checks** - Consider adding a lint rule for common violations
3. **Standards evolution** - Update standards as new patterns emerge

---

## Appendix: Quick Reference

### The "Should" Pattern

**Use for:**
- Expected states: `"Flag should be true"`
- Value comparisons: `"should be 'scan'"`
- Type checks: `"should be a list"`

**Don't use for:**
- Operation failures (use "Failed to")
- Invariants (use "must")

### Standard Message Structure

**Order:** [Subject] + [Expected State] + [Context]

```
✅ "Daemon should be healthy after boot"
✅ "Projects should be a list"
✅ "healthz should return 200"
```

### Common Patterns

| Scenario | Pattern | Example |
|----------|---------|---------|
| Expected behavior | `<subject> should <state>` | `"Daemon should start"` |
| Operation failure | `Failed to <action> <target>` | `"Failed to read config"` |
| Invariant | `<subject> must <condition>` | `"projects.rs must exist"` |
| Value comparison | `Expected <expected>, got <actual>` | `"Expected string, got integer"` |

---

## Conclusion

HOOP's error message consistency issues are **significant but fixable**. The problems are well-understood, standards are defined, and most fixes are straightforward. Priority 1 fixes (sentence case, "should" pattern) can be implemented in 2-3 hours with mostly automated changes, improving compliance from 33.8% to 75%+.

**Key Takeaway:** Investing in error message consistency will:
- Reduce debugging time for all developers
- Improve code maintainability
- Help new developers onboard faster
- Make the codebase more professional

**Next Step:** Create a bead to implement Priority 1 fixes and track progress with periodic validation re-runs.

---

**Report Status:** Complete  
**Version:** 1.0  
**Last Updated:** 2026-08-12  
**Validation Data:** from bead bf-55am0  
**Standards Reference:** from bead bf-4vtp7
