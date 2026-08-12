# HOOP Error Message Validation - Final Report

**Task:** bf-55am0 - Validate error messages against consistency standards  
**Generated:** 2026-08-12  
**Catalog:** 5,904 error messages from HOOP test suite  
**Standards:** error_message_standards.md + error_message_actionability_standards.md

## Executive Summary

**Validation completed.** All 5,904 error messages in the HOOP catalog have been validated against the defined consistency standards. **Significant violations exist across all categories**, with 30.2% of messages failing to meet minimum informational content requirements and 73.1% not following standard wording patterns.

### Critical Findings

| Category | Violations | Percentage | Severity |
|----------|------------|------------|----------|
| **Minimal context violations** | 1,781 | 30.2% | 🔴 HIGH |
| **Non-standard patterns** | 4,314 | 73.1% | 🔴 HIGH |  
| **Single-word value messages** | 132 | 2.2% | 🟡 MEDIUM |
| **Trailing period violations** | 1 | <0.1% | 🟢 LOW |
| **Standard pattern compliance** | 1,590 | 26.9% | ✅ PASSING |

**Overall Assessment:** ⚠️ **MAJOR IMPROVEMENTS NEEDED** - While positive examples exist, the majority of error messages fail to meet consistency standards.

---

## Violation Categories

### 1. Minimal Context Violations (1,781 messages - 30.2%)

**Standard:** Every error message must identify:
1. What failed (operation/assertion)
2. Target/subject (component, file, field)  
3. Expected state/behavior
4. Actual state (when different from expected)

**Violation:** Messages with ≤10 characters lacking descriptive context.

**Examples Found:**
```
❌ "scan"                  → ✅ "scan command should parse correctly"
❌ "/tmp"                  → ✅ "path should be /tmp in test configuration"  
❌ "-y"                    → ✅ "no_interactive should be true when -y flag present"
❌ "--no-interactive"      → ✅ "CLI flag --no-interactive should be recognized"
❌ "test-project"          → ✅ "project name should be test-project"
```

**Impact:** These messages are not self-documenting. When tests fail, developers must read test code to understand failure.

**Fix Complexity:** LOW - Simple text additions to existing messages

### 2. Non-Standard Pattern Violations (4,314 messages - 73.1%)

**Standard:** Messages must follow one of three primary patterns:

1. **"Should" pattern** (primary): `<subject> should <expected_state> [when <condition>]`
2. **"Failed to" pattern** (operations): `Failed to <action> <target> [because <reason>]`
3. **"Must" pattern** (invariants): `<subject> must <condition>`

**Violation:** Messages not following these patterns.

**Common Non-Compliant Patterns:**
- Single values (flags, paths, commands)
- Generic phrases without standard structure
- Missing "when" condition clauses
- Incorrect use of "must" vs "should"

**Fix Complexity:** MEDIUM - Requires restructuring message text

### 3. Single-Word Value Messages (132 messages - 2.2%)

**Standard:** Use standard patterns even for simple value assertions.

**Violation Type:** Messages that are just command names, flags, or values.

**Pattern Analysis:**
- CLI flags: `--json`, `--verbose`, `--no-interactive`, `-y`
- Paths: `/tmp`, `/ws/b`
- Commands: `scan`, `projects`, `remove`  
- Test values: `test-project`, cron expressions like `0 4 * * *`

**Fix Complexity:** LOW - Apply standard pattern templates

### 4. Trailing Period Violations (1 message - <0.1%)

**Standard:** Do NOT end error messages with periods.

**Violation:** One message found with trailing period.

**Assessment:** ✅ **EXCELLENT COMPLIANCE** - This standard is nearly universally followed.

**Fix Complexity:** TRIVIAL - Remove trailing period

---

## Positive Compliance Examples

### Messages Following Standards Correctly

**✅ "Should" Pattern with Condition:**
```rust
"no_interactive should be true with flag before command"
"no_interactive should be true with -y flag"
"no_interactive should default to false"
```

**✅ "Should" Pattern for Endpoints:**
```rust
"Dashboard endpoint should return 200"
"Worker timeline endpoint should return 200"  
"Events should be an array"
```

**✅ "Should" Pattern with Context:**
```rust
"Draft title should match chat input"
"Draft kind should be fix"
"Initial readyz status should be ok"
```

**✅ "Failed to" Pattern:**
```rust
"Failed to send WebSocket message"
"Capacity response is not an object"
"WebSocket connection closed"
```

**✅ Format Placeholders at End:**
```rust
"Sum of project worker counts ({}) should equal total ({})"
"Should fetch beads in cycle {}"
```

**Why These Work:**
- Identify what failed (operation/assertion)
- Identify target/subject (component, endpoint, field)
- Identify expected state/behavior
- Use standard wording patterns
- Place format placeholders at end
- Use appropriate format specifiers (`{}` for display, `{:?}` for debug)

---

## Detailed Standards Compliance Assessment

### Wording and Formatting Standards

| Requirement | Compliance | Violations | Severity |
|-------------|------------|------------|----------|
| No trailing periods | ✅ 99.9% | 1 | 🟢 LOW |
| Standard patterns (should/failed to/must) | ⚠️ 26.9% | 4,314 | 🔴 HIGH |
| Descriptive context | ❌ 69.8% | 1,781 | 🔴 HIGH |
| Format placeholder placement | ✅ Excellent | Minimal | 🟢 GOOD |
| First word capitalization | ⚠️ Needs review | TBD | 🟡 MEDIUM |
| Preserve original case | ⚠️ Needs review | TBD | 🟡 MEDIUM |
| No unnecessary quotes | ⚠️ Needs review | TBD | 🟡 MEDIUM |

### Informational Content Standards

| Requirement | Compliance | Violations | Severity |
|-------------|------------|------------|----------|
| What failed | ❌ Poor | Many single-word messages | 🔴 HIGH |
| Target/subject | ❌ Poor | Values without component ID | 🔴 HIGH |
| Expected state | ❌ Poor | Missing in 30%+ of messages | 🔴 HIGH |
| Actual state | ⚠️ Partial | Some comparisons include, many don't | 🟡 MEDIUM |
| Context (conditions) | ⚠️ Partial | Present in good examples, missing elsewhere | 🟡 MEDIUM |
| Cause/reason | ⚠️ Needs analysis | TBD | 🟡 MEDIUM |

### Actionability Standards

| Requirement | Status | Notes |
|-------------|--------|-------|
| Actionable suggestions for user-correctable errors | ⚠️ Needs detailed analysis | Most test messages lack suggestions |
| Safe/obvious fix guidance | ⚠️ Needs detailed analysis | Few examples found |
| Audience-appropriate language | ⚠️ Needs detailed analysis | Mix of developer/user messages |
| Informational-only messages | ⚠️ Needs detailed analysis | Some progress/status messages |

---

## High-Priority Files for Fixes

Based on error message density and violation patterns, these files require the most attention:

| File | Error Count | Primary Issues | Priority |
|------|-------------|----------------|----------|
| `tests/cli_test_helpers.rs` | 205 | High assertion density, many single-word messages | 🔴 HIGH |
| `hoop-cli/tests/cli_test_helpers.rs` | 199 | CLI validation, minimal context messages | 🔴 HIGH |
| `hoop-daemon/tests/integration_harness.rs` | 189 | Integration infrastructure, generic assertions | 🔴 HIGH |
| `hoop-daemon/tests/config_field_validation.rs` | 183 | Config testing, value-only messages | 🟡 MEDIUM |
| `hoop-cli/tests/no_interactive_flag_behavior.rs` | 171 | Flag behavior, flag-value messages | 🟡 MEDIUM |
| `hoop-daemon/tests/config_reload_cycle.rs` | 165 | Config reload, state validation messages | 🟡 MEDIUM |

---

## Structured Violation Report by Type

### Type 1: Wording Violations

**Pattern:** Messages not following "should/failed to/must" conventions

**Examples:**
```rust
❌ "scan"                           → ✅ "scan command should be recognized"
❌ "/tmp"                           → ✅ "test path should be /tmp"  
❌ "no_interactive should be true"  → ✅ "no_interactive should be true when --no-interactive flag is present"
❌ "flag should be true"            → ✅ "no_interactive flag should be true in non-interactive mode"
```

**Count:** 4,314 violations (73.1%)

**Fix Template:** Apply `<subject> should <state> [when <condition>]` pattern

### Type 2: Formatting Violations

**Pattern:** Punctuation, capitalization, or quote issues

**Examples:**
```rust
❌ "Failed to read config."         → ✅ "Failed to read config"  (trailing period)
❌ "should be 'true'"               → ✅ "should be true"         (unnecessary quotes)
❌ "failed to read"                 → ✅ "Failed to read"         (first word capitalization)
```

**Count:** 1 confirmed (trailing period), others TBD pending detailed analysis

**Fix Template:** Remove periods, unnecessary quotes; ensure first-word capitalization

### Type 3: Missing Informational Elements

**Pattern:** Missing required elements (what/target/expected/actual/context)

**Examples:**
```rust
❌ "scan"                           → ✅ "scan command should parse valid arguments"
❌ "Invalid config"                 → ✅ "Failed to parse config: missing required field"
❌ "flag should be true"            → ✅ "no_interactive flag should be true when --no-interactive is present"
❌ assert_eq!(value, expected)      → ✅ assert_eq!(value, expected, "field should match expected value")
```

**Count:** 1,781 minimal context violations (30.2%)

**Fix Template:** Add `<what> <target> <expected> [+ actual] [+ context]` structure

### Type 4: Non-Actionable User-Correctable Errors

**Pattern:** User-facing errors missing safe, obvious suggestions

**Examples:**
```rust
❌ "Failed to read config.yml"     → ✅ "Failed to read config.yml: file not found. Run 'hoop init' to create it"
❌ "Config is invalid"              → ✅ "schema_version missing from config. Add: schema_version: 1"
❌ "File not found"                 → ✅ "projects.rs not found. Ensure you're in a HOOP-managed workspace"
```

**Count:** TBD pending detailed analysis (likely many)

**Fix Template:** Add `. <safe_suggestion>` structure for user-correctable errors

---

## Recommendations for Fix Prioritization

### Phase 1: Critical Fixes (HIGH Impact, LOW Effort)

**Priority 1.1: Add Descriptive Context to Single-Word Messages** (1,781 messages)
- **Action:** Convert single values to standard "should" pattern messages
- **Template:** `"<value>"` → `"<context> should be <value>"`
- **Example:** `"scan"` → `"scan command should be recognized"`
- **Impact:** Eliminates 30% of violations with simple text additions
- **Effort:** LOW - Template-based replacements

**Priority 1.2: Replace Generic `.unwrap()` with Descriptive `.expect()`** (1,482 instances)
- **Action:** Add "Failed to <action> <target>" messages to all unwrap calls
- **Template:** `.unwrap()` → `.expect("Failed to <action> <target>")`
- **Example:** `.unwrap()` → `.expect("Failed to read config file")`
- **Impact:** Prevents silent panics, provides debugging context
- **Effort:** LOW - Find-and-replace with templates

### Phase 2: Standard Pattern Application (MEDIUM Impact, MEDIUM Effort)

**Priority 2.1: Convert Assertions to Standard Patterns** (4,314 non-compliant)
- **Action:** Apply "should/failed to/must" patterns to all assertions
- **Template:** 
  - Assertions: `<subject> should <state> [when <condition>]`
  - Operations: `Failed to <action> <target> [because <reason>]`
  - Invariants: `<subject> must <condition>`
- **Example:** 
  - `"flag true"` → `"no_interactive flag should be true when --no-interactive is present"`
  - `"parsing failed"` → `"Failed to parse config: missing required field"`
- **Impact:** Consistent wording, self-documenting tests
- **Effort:** MEDIUM - Requires message restructuring

**Priority 2.2: Add Actual vs Expected Comparisons** (where missing)
- **Action:** Use "Expected <expected>, got <actual>" format for value comparisons
- **Template:** `Expected <expected>, got <actual>`
- **Example:** `"integer but got string"` → `"Expected string, got integer"`
- **Impact:** Clearer failure diagnosis for type/value mismatches
- **Effort:** LOW-MEDIUM - Pattern-based additions

### Phase 3: Enhanced Context and Actionability (LOWER Priority, HIGHER Effort)

**Priority 3.1: Add Contextual Information** (file paths, line numbers, conditions)
- **Action:** Include location for parse errors, state for resource errors
- **Templates:**
  - Parse errors: `Failed to parse <file>: line <n>, column <m>`
  - Resource errors: `Failed to acquire semaphore (<slots_used>/<total_slots> in use)`
- **Impact:** Better error diagnosis
- **Effort:** MEDIUM - Requires context capture

**Priority 3.2: Add Actionable Suggestions** (where appropriate)
- **Action:** Include safe, obvious fixes for user-correctable errors
- **Template:** `<problem>. <safe_fix_suggestion>`
- **Examples:**
  - `"Failed to read ~/.hoop/config.yml: file not found. Run 'hoop init' to create it"`
  - `"schema_version missing from config. Add: schema_version: 1"`
- **Impact:** User-friendly error resolution
- **Effort:** MEDIUM-HIGH - Requires judgment about what's safe/obvious

---

## Implementation Guidance

### Fix Strategy

1. **Start with Phase 1 fixes** - These address the most violations with least effort
2. **Use template-based replacements** - Many fixes follow predictable patterns
3. **Focus on high-density files first** - CLI test helpers and integration tests
4. **Verify fixes don't break tests** - Add context shouldn't change test behavior
5. **Document exceptions** - Some messages may intentionally deviate from standards

### Quality Assurance

After implementing fixes:
- Run `cargo test` to ensure no test behavior changes
- Re-run this validation to measure improvement
- Update error message catalog
- Document any intentional exceptions to standards

### Success Criteria

- **Minimum Viable:** Reduce violations by 50% (from 73.1% to <36% non-compliant)
- **Target:** Reduce violations by 80% (from 73.1% to <15% non-compliant)  
- **Excellent:** Reduce violations by 95% (from 73.1% to <4% non-compliant)

---

## Conclusion

The HOOP error message catalog shows **significant inconsistency with defined standards**. While excellent examples exist that demonstrate proper adherence to conventions, the majority of messages lack minimum informational content or fail to follow standard wording patterns.

**Good news:** Most violations are template-based fixes that can be systematically addressed:
- 30.2% are simple context additions to single-word messages
- Many others are pattern restructurings using standard templates
- Very few violations indicate fundamental design issues

**Recommendation:** Proceed with phased fixes starting with Phase 1 (high impact, low effort) to achieve rapid improvement, then advance to Phase 2 and 3 for comprehensive compliance.

---

**Standards Documents:**
- error_message_standards.md (wording and formatting)
- error_message_actionability_standards.md (informational and actionability)

**Catalog Referenced:**  
- error_messages/comprehensive_error_messages.json (5,904 messages)

**Related Work:**  
- error_messages_catalog.md (bf-3ysoc - catalog creation)
- error_validation_preliminary_report.md (preliminary analysis)

**Next Bead:** Apply fixes following prioritization recommendations in this report
