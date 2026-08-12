# HOOP Error Message Validation - Preliminary Report

**Generated:** 2026-08-12  
**Task:** bf-55am0 - Validate error messages against consistency standards  
**Status:** In Progress - Comprehensive agent analysis underway

## Executive Summary

Based on preliminary analysis of the HOOP error message catalog (5,904 total messages), **significant violations of consistency standards are evident**. While some messages follow established patterns, many lack the minimum required informational content and formatting specified in the standards documents.

### Key Findings

| Metric | Count | Percentage | Assessment |
|--------|-------|------------|------------|
| Total error messages | 5,904 | 100% | Baseline |
| Messages with ≤10 characters | 1,781 | 30.2% | **Major violation** - insufficient context |
| Messages following standard patterns | 1,590 | 26.9% | Partial compliance |
| Messages with trailing periods | 1 | <0.1% | Minor violation |

## Critical Violations Identified

### 1. Minimal Context Violations (1,781 messages - 30.2%)

**Issue:** Messages containing only values or flags without descriptive context.

**Examples found:**
```
Message: "scan"                  (should be: "scan command should parse correctly")
Message: "/tmp"                  (should be: "path should be /tmp in test configuration")  
Message: "-y"                    (should be: "no_interactive should be true when -y flag present")
Message: "--no-interactive"      (should be: "CLI flag --no-interactive should be recognized")
Message: "test-project"          (should be: "project name should be test-project")
```

**Standard violation:** Fails minimum informational content requirement - missing:
- What failed (operation/assertion)
- Expected state/behavior  
- Context (conditions, purpose)

**Impact:** These messages are not self-documenting. When tests fail, developers must read test code to understand what went wrong.

### 2. Single-Word Value Messages (132 messages)

**Issue:** Messages that are just command names, flags, or values.

**Pattern analysis:**
- CLI flags: `--json`, `--verbose`, `--no-interactive`, `-y`
- Paths: `/tmp`, `/ws/b`
- Commands: `scan`, `projects`, `remove`
- Test values: `test-project`, cron expressions

**Standard violation:** Fails wording conventions - should use:
- `<subject> should <state> [when <condition>]` pattern
- `<action> should <result>` pattern for validation tests
- `Expected <expected>, got <actual>` for value comparisons

### 3. Missing Standard Patterns (4,314 messages - 73.1%)

**Issue:** Majority of messages do not follow the three primary patterns defined in standards.

**Standard patterns required:**
1. **"Should" pattern** (primary): `<subject> should <expected_state> [when <condition>]`
2. **"Failed to" pattern** (operations): `Failed to <action> <target> [because <reason>]`
3. **"Must" pattern** (invariants): `<subject> must <condition>`

**Compliance:** Only 1,590 messages (26.9%) follow these patterns.

## Positive Patterns Found

### Good Examples (Following Standards)

Some messages demonstrate proper adherence to standards:

```rust
// ✅ Correct - "should" pattern with condition
"no_interactive should be true with flag before command"
"no_interactive should be true with -y flag"
"no_interactive should default to false"

// ✅ Correct - "should" pattern for endpoints  
"Dashboard endpoint should return 200"
"Worker timeline endpoint should return 200"
"Events should be an array"

// ✅ Correct - "should" pattern with context
"Draft title should match chat input"
"Draft kind should be fix"
"Initial readyz status should be ok"

// ✅ Correct - "Failed to" pattern
"Failed to send WebSocket message"
"Capacity response is not an object"
"WebSocket connection closed"

// ✅ Correct - Format placeholders at end
"Sum of project worker counts ({}) should equal total ({})"
"Should fetch beads in cycle {}"
```

**These messages:**
- Identify what failed (operation/assertion)
- Identify target/subject (component, endpoint, field)
- Identify expected state/behavior
- Use standard wording patterns
- Place format placeholders at end
- Use appropriate format specifiers (`{}` for display, `{:?}` for debug)

## Standards Compliance Assessment

### Wording and Formatting Standards

| Requirement | Status | Notes |
|-------------|--------|-------|
| No trailing periods | ✅ Excellent | Only 1 violation found |
| Standard patterns (should/failed to/must) | ⚠️ Partial | 27% compliance |
| Descriptive context | ❌ Poor | 30% too brief |
| Format placeholder placement | ✅ Good | Examples show proper end placement |
| Capitalization conventions | ⚠️ Unknown | Needs analysis |

### Informational Content Standards

| Requirement | Status | Notes |
|-------------|--------|-------|
| What failed | ❌ Poor | Many single-word messages lack operation context |
| Target/subject | ❌ Poor | Values without component identification |
| Expected state | ❌ Poor | Missing in 30%+ of messages |
| Actual state | ⚠️ Partial | Some comparisons include, many don't |
| Context (conditions) | ⚠️ Partial | Present in good examples, missing elsewhere |
| Cause/reason | ⚠️ Unknown | Needs detailed analysis |

### Actionability Standards

| Requirement | Status | Notes |
|-------------|--------|-------|
| Actionable suggestions for user-correctable errors | ⚠️ Unknown | Needs detailed analysis |
| Safe/obvious fix guidance | ⚠️ Unknown | Needs detailed analysis |
| Audience-appropriate language | ⚠️ Unknown | Needs detailed analysis |

## Files Requiring Priority Attention

Based on message density and violation patterns, these files likely need the most work:

1. **tests/cli_test_helpers.rs** (205 errors) - High assertion density
2. **hoop-cli/tests/cli_test_helpers.rs** (199 errors) - CLI validation focus
3. **hoop-daemon/tests/integration_harness.rs** (189 errors) - Integration infrastructure
4. **hoop-daemon/tests/config_field_validation.rs** (183 errors) - Config testing
5. **hoop-cli/tests/no_interactive_flag_behavior.rs** (171 errors) - Flag behavior

## Recommendations for Fix Prioritization

### Phase 1: Critical Fixes (High Impact, Low Effort)

1. **Add descriptive context to single-word messages** (1,781 messages)
   - Add `<subject> should <state>` pattern to value assertions
   - Example: `"scan"` → `"scan command should be recognized"`
   - Impact: Eliminates 30% of violations with simple text additions

2. **Replace generic `.unwrap()` with descriptive `.expect()`** (1,482 instances)
   - Add "Failed to <action> <target>" messages
   - Example: `.unwrap()` → `.expect("Failed to read config file")`
   - Impact: Prevents silent panics, provides debugging context

### Phase 2: Standard Pattern Application (Medium Impact, Medium Effort)

3. **Convert assertions to standard patterns** (4,314 non-compliant)
   - Apply "should/failed to/must" patterns to all assertions
   - Add condition context with "when" clauses
   - Impact: Consistent wording, self-documenting tests

4. **Add actual vs expected comparisons** (where missing)
   - Use "Expected <expected>, got <actual>" format
   - Include actual values in debug format (`{:?}`)
   - Impact: Clearer failure diagnosis

### Phase 3: Enhanced Context and Actionability (Lower Priority, Higher Effort)

5. **Add contextual information** (file paths, line numbers, conditions)
   - Include location for parse errors
   - Add state for resource errors
   - Impact: Better error diagnosis

6. **Add actionable suggestions** (where appropriate)
   - Include safe, obvious fixes for user-correctable errors
   - Example: "file not found. Run 'hoop init' to create it"
   - Impact: User-friendly error resolution

## Next Steps

1. **Await comprehensive agent analysis** for detailed violation categorization
2. **Create structured violation database** with per-message analysis
3. **Generate fix templates** for each violation category
4. **Prioritize fixes** by impact and effort
5. **Execute phased improvements** following recommended order

---

**Analysis Status:** Preliminary findings complete. Comprehensive agent analysis in progress.  
**Standards Documents Referenced:**
- error_message_standards.md (wording and formatting)
- error_message_actionability_standards.md (informational and actionability)  
**Catalog Referenced:** error_messages/comprehensive_error_messages.json (5,904 messages)
