# HOOP Error Message Validation Report

**Generated:** 2026-08-12  
**Bead:** bf-55am0  
**Catalog Source:** bf-3ysoc  
**Standards Source:** bf-4vtp7  

---

## Executive Summary

This report validates all 5,904 error messages from the HOOP test suite against the defined consistency standards in `docs/error-message-standards.md`.

### Overall Assessment

**Status:** ❌ NEEDS IMPROVEMENT

- **Total Messages Analyzed:** 5,904
- **Total Violations Found:** 3,910
- **Compliance Rate:** 33.8%
- **Good Examples:** 949 (16.1%)

### Violation Breakdown

| Category | Count | Percentage | Severity |
|----------|-------|------------|----------|
| **Wording Issues** | 2,677 | 45.3% | High |
| **Not Actionable** | 648 | 11.0% | Medium |
| **Missing Information** | 585 | 9.9% | Medium |
| **Formatting Issues** | 0 | 0.0% | Low |
| **Good Examples** | 949 | 16.1% | N/A |

---

## Detailed Violations by Category

### 1. Wording Issues (2,677 violations - 45.3%)

#### 1.1 Incorrect Case (1,885 instances)

**Issue:** Messages don't follow sentence case conventions (first word capitalized, rest lowercase except proper nouns).

**Examples of violations:**
```rust
❌ "no_interactive should be true with flag before command"
❌ "no_interactive should be true with -y flag"
❌ "no_interactive should default to false"
```

**Standard:**
```
✅ "No_interactive should be true with flag before command"
✅ "No_interactive should be true with -y flag"
✅ "No_interactive should default to false"
```

**Impact:** High - Inconsistent capitalization makes error messages harder to scan and appear unprofessional.

**Fix Pattern:**
1. Capitalize first word of message
2. Ensure remaining words are lowercase (except proper nouns like "HOOP", "HTTP", "WebSocket")

---

#### 1.2 Missing "Should" Pattern (791 instances)

**Issue:** Assertion and validation messages don't use the standard "should {verb}" pattern.

**Examples of violations:**
```rust
❌ "arg1"
❌ "Some tests failed: {:?}"
❌ "test-project"
```

**Standard:**
```
✅ "First argument should be 'scan'"
✅ "All tests should pass"
✅ "Project name should be 'test-project'"
```

**Impact:** High - Missing "should" pattern makes messages less descriptive and harder to understand.

**Fix Pattern:**
- For assertions: Add "should {verb}" to describe expected state
- For comparisons: Use "expected {value}, got {actual}"
- For validation: Use "should {fail/pass/succeed}"

---

#### 1.3 Trailing Period (1 instance)

**Issue:** Message ends with a period, violating the no-trailing-period standard.

**Example:**
```rust
❌ "Phase 2 exit gate FAILED: {} of 13 core deliverables lack passing tests. \
    Marquee features (14-17) cannot merge until all core deliverables are verified."
```

**Standard:**
```
✅ "Phase 2 exit gate FAILED: {} of 13 core deliverables lack passing tests \
    Marquee features (14-17) cannot merge until all core deliverables are verified"
```

**Impact:** Low - Only one instance found, but violates standard convention.

**Fix Pattern:**
- Remove trailing period from all error messages

---

### 2. Not Actionable (648 violations - 11.0%)

**Issue:** Messages are vague and don't suggest what went wrong or what the expected state should be.

**Examples of violations:**
```rust
❌ "test-project"
❌ "No arguments provided"
❌ "new-project"
```

**Standard:**
```
✅ "Project should be 'test-project'"
✅ "Command should require at least one argument"
✅ "Project name should be 'new-project'"
```

**Impact:** Medium - Vague messages waste developer time during debugging.

**Fix Pattern:**
- Add context about what's being validated
- Describe the expected state
- Include relevant identifiers (field names, endpoints, etc.)

---

### 3. Missing Information (585 violations - 9.9%)

**Issue:** Messages lack sufficient context for debugging (identifiers, field paths, endpoints).

**Examples of violations:**
```rust
❌ "arg1"
❌ "Some tests failed: {:?}"
❌ Missing file context in read operations
```

**Standard:**
```
✅ "First argument should be 'scan', got 'status'"
✅ "Tests in suite {} should all pass: {:?}"
✅ "Failed to read {}: file not found or permission denied"
```

**Impact:** Medium - Developers can't identify the source of failures without additional debugging.

**Fix Pattern:**
- Include identifiers: bead IDs, project names, worker names
- Include field paths: `"agent.adapter"`, `"metrics.enabled"`
- Include HTTP endpoints: `"GET /api/beads"`, `"/healthz"`
- Include connection/thread context: `"(conn {})"`, `"(worker {})"`

---

### 4. Formatting Issues (0 violations - 0.0%)

**Status:** ✅ PASS

No formatting issues found. Quote usage appears correct:
- Strings and field names are properly quoted
- Types and booleans are not incorrectly quoted
- Context brackets are used appropriately

---

## Files with Most Violations

### Top 10 Files by Violation Count

| Rank | File | Violations | Top Issues |
|------|------|------------|-------------|
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

## Good Examples to Follow

### Excellent Compliance Examples (949 total)

These messages follow all standards correctly:

```rust
✅ "Should parse with flag before command"
✅ "Should parse with flag after command"
✅ "Should parse with -y flag"
✅ "Should parse without flag"
✅ "Flag should be true at any position"
✅ "Flag position should not affect value for {}"
```

**Pattern:** These examples:
- Use proper sentence case
- Include "should" pattern
- Are descriptive and actionable
- Provide context about what's being tested

---

## Recommendations

### Priority 1: High Impact, Quick Wins

1. **Fix Sentence Case (1,885 instances)**
   - Capitalize first word of all messages
   - Ensure remaining words are lowercase
   - Exception: Proper nouns (HOOP, HTTP, WebSocket)

2. **Add "Should" Pattern (791 instances)**
   - Convert bare values to descriptive assertions
   - Use "should {verb}" pattern for expected states
   - Example: `"scan"` → `"Command should be 'scan'"`

3. **Remove Trailing Period (1 instance)**
   - Quick fix: Remove period from phase gate message

**Estimated Effort:** 2-3 hours for automated find-replace with manual review

---

### Priority 2: Medium Impact

1. **Add Context to Vague Messages (648 instances)**
   - Include what's being validated
   - Add relevant identifiers (field names, endpoints)
   - Describe expected state clearly

2. **Include Debugging Context (585 instances)**
   - Add bead IDs, project names, worker names where relevant
   - Include field paths for config validation errors
   - Add HTTP endpoint names for API errors

**Estimated Effort:** 4-6 hours for targeted improvements

---

### Priority 3: Lower Priority (Nice to Have)

1. **Consolidate Similar Patterns**
   - Identify messages with similar meaning but different wording
   - Standardize on best-of-breed phrasing
   - Create message templates for common scenarios

2. **Enhance Actionability**
   - Add "why" context for non-obvious assertions
   - Suggest next steps for validation errors
   - Include fix hints when appropriate

**Estimated Effort:** 2-3 hours for pattern consolidation

---

## Implementation Strategy

### Phase 1: Automated Fixes (Priority 1)

**Tools:** `sed`, `rg` (ripgrep), or custom script

**Tasks:**
1. Capitalize first letter of all error messages
2. Add "should" pattern to bare value assertions
3. Remove trailing periods

**Validation:** Run validation script after each batch

---

### Phase 2: Manual Improvements (Priority 2)

**Approach:** File-by-file review of high-impact test files

**Priority Files:**
1. `hoop-daemon/tests/draft_queue_invariants.rs` (205 violations)
2. `tests/cli_test_helpers.rs` (159 violations)
3. `hoop-daemon/tests/multi_operator_concurrency.rs` (136 violations)

**Tasks:**
1. Add context identifiers (IDs, names, paths)
2. Improve vague messages with descriptive text
3. Include HTTP endpoints and field paths

---

### Phase 3: Pattern Consolidation (Priority 3)

**Approach:** Identify and standardize similar patterns

**Tasks:**
1. Extract all messages, group by similarity
2. Vote on best phrasing for each pattern
3. Create message template library
4. Update code to use templates

---

## Quality Metrics

### Current State

- **Compliance Rate:** 33.8%
- **Violations per 100 messages:** 66.2
- **Top Violation:** Incorrect case (48.2% of all violations)

### Target State (Post-Fix)

- **Compliance Rate:** 90%+
- **Violations per 100 messages:** <10
- **Top Violation:** None (all categories <5%)

---

## Appendix: Validation Methodology

### Standards Reference

All validation performed against `docs/error-message-standards.md` (bead bf-4vtp7):

1. **Wording Conventions** (§2)
   - Sentence case, no trailing period
   - "should {verb}" pattern
   - Article usage ("a", "an", "the")

2. **Formatting Patterns** (§3)
   - Quote usage (strings vs. types)
   - Context brackets
   - Multi-line messages

3. **Informational Requirements** (§4)
   - Required info per error type
   - Context (identifiers, field paths, endpoints)
   - Type information

4. **Actionability Guidelines** (§5)
   - Suggest expected state
   - Explain why
   - Provide next steps

### Validation Script

```bash
# Re-run validation
python3 scripts/validate_error_messages.py
```

Output generates this report with:
- Per-message violation checking
- Categorization by type
- File-level summaries
- Sample violations for review

---

## Next Steps

1. **Review this report** with bead bf-55am0 closed
2. **Create fix bead** (recommended: `bf-XXXXX`) to implement Priority 1 fixes
3. **Track progress** with periodic validation re-runs
4. **Update standards** if patterns emerge that aren't covered

---

**Validation Complete**

This report provides a complete inventory of all error message consistency violations across the HOOP test suite. Use it as the roadmap for systematic error message improvement work.
