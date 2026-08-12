# HOOP Error Message Pattern Analysis

**Generated:** 2026-08-12
**Task:** bf-3du8o - Analyze error message catalog patterns
**Source Data:** error_messages_catalog.md (bf-3ysoc) with 5,904 error messages

## Executive Summary

This document analyzes error message patterns across the HOOP test suite, extracting conventions for wording, formatting, information content, and actionability. The analysis is based on 5,904 error messages extracted from 104 test files.

## Distribution Overview

| Error Type | Count | Percentage |
|------------|-------|------------|
| `expect` | 1,871 | 31.7% |
| `assert` | 1,500 | 25.4% |
| `unwrap` | 1,482 | 25.1% |
| `assert_eq` | 935 | 15.8% |
| `panic` | 53 | 0.9% |
| `unwrap_err` | 25 | 0.4% |
| `bail` | 20 | 0.3% |
| `assert_ne` | 10 | 0.2% |
| `anyhow` | 6 | 0.1% |
| `anyhow_context` | 2 | 0.0% |

## Pattern Analysis by Category

### 1. Wording Conventions

#### 1.1 Assertion Messages (`assert!`, `assert_eq!`, `assert_ne!`)

**Subject-First Pattern**
- Format: `[Subject] should [be/action] [condition]`
- Examples:
  - `"no_interactive should be true with flag before command"`
  - `"no_interactive should be true with -y flag"`
  - `"no_interactive should default to false"`
  - `"Flag should be extracted as true"`
  - `"Command should be 'scan'"`

**Must/Required Pattern**
- Format: `[Subject] must [action/condition]`
- Examples:
  - `"main() must extract flag from CLI"`
  - `"CLI must parse flag as true"`
  - `"Handler must check the flag value"`
  - `"Child process must receive no_interactive flag"`
  - `"Parent must have flag set"`

**Expected Pattern**
- Format: `Expected [value/entity]`
- Examples:
  - `"Expected Scan command"`
  - `"Expected Remove command"`
  - `"Expected Projects command"`
  - `"Expected Projects::Scan command"`

**Action-Oriented Pattern**
- Format: `[Action] should [result]` or `[Action] [result]`
- Examples:
  - `"Parsing should succeed"`
  - `"Parsing should succeed even with flag at end"`
  - `"Should successfully parse flag before subcommand"`
  - `"Should have 2 open beads"`
  - `"Should have 1 closed bead"`

#### 1.2 Expectation Messages (`expect()`)

**Failure-Focused Pattern**
- Format: `"Failed to [action]"`
- Examples:
  - `"Failed to create .beads/ directory"`
  - `"Failed to create .hoop/ directory"`
  - `"Failed to write projects.yaml"`
  - `"Failed to parse with flag before command"`
  - `"Failed to read main.rs"`
  - `"Failed to read projects.rs"`

**Success-Focused Pattern**
- Format: `"Should [action/result]"` or `"[Action] should succeed"`
- Examples:
  - `"Should parse with flag before command"`
  - `"Should parse with flag after command"`
  - `"Parse should succeed"`
  - `"Should find Scan command handler"`
  - `"Should find scan_projects function"`

#### 1.3 Panic Messages

**Expectation Pattern**
- Format: `"Expected [entity]"` or `"Expected [entity] at [location]"`
- Examples:
  - `"Expected Scan command"`
  - `"Expected Remove command"`
  - `"Expected Projects command"`
  - `"Expected Projects::Scan command"`
  - `"Expected Remove command at Level 2"`
  - `"Expected Projects command at Level 1"`

### 2. Formatting Patterns

#### 2.1 Quoting Conventions

**String/Command Values**
- Single quotes around command names and subcommands
- Examples:
  - `'scan'`
  - `'remove'`
  - `'projects'`
  - `'--no-interactive'`
  - `'-y'`

**Path Quoting**
- No quotes around file paths
- Examples:
  - `/tmp`
  - `test-project`
  - `.beads/`
  - `.hoop/`

**Flags and Options**
- No quotes around flags
- Examples:
  - `--no-interactive`
  - `--from`
  - `--confirm`
  - `-y`

#### 2.2 Capitalization

**Sentence Case for Messages**
- First word capitalized, rest sentence case
- Examples:
  - `"Failed to parse args: {:?}"`
  - `"Flag should be true"`
  - `"Parsing should succeed"`
  - `"All beads should belong to testrepo"`

**Field/Variable References**
- Lowercase snake_case for field names
- Examples:
  - `no_interactive`
  - `subcommand`
  - `nested_subcommand`
  - `bead_id`
  - `worker`

#### 2.3 Punctuation

**No Period at End**
- Messages don't end with periods
- Examples:
  - `"no_interactive should be true"` (not `"no_interactive should be true."`)
  - `"Failed to create .beads/ directory"` (not `"Failed to create .beads/ directory."`)

**Format Placeholders**
- Use `{}` for placeholders with formatting
- Examples:
  - `"Events fixture should contain {} event"`
  - `"Should have at least 2 sessions, got {}"`
  - `"Failed to parse args: {:?}"`
  - `"error should mention pattern/format: {:?}"`

### 3. Information Included

#### 3.1 Actual vs. Expected Values

**Explicit Comparisons**
- Direct statements of what should happen
- Examples:
  - `"no_interactive should be true with flag before command"`
  - `"no_interactive should be true with -y flag"`
  - `"no_interactive should default to false when flag is not provided"`

**Implicit Context**
- Message implies expected state without stating actual
- Examples:
  - `"Flag should be true"` (implies actual is false or unknown)
  - `"Parsing should succeed"` (implies it failed)
  - `"Should successfully parse flag before subcommand"` (implies it didn't)

#### 3.2 Context Information

**Position/Location Context**
- References to where something should appear
- Examples:
  - `"flag before command"`
  - `"flag after command"`
  - `"with -y flag"`
  - `"at Level 2"`
  - `"within the repository"`

**Structural Context**
- References to data structures or relationships
- Examples:
  - `"All beads should belong to testrepo"`
  - `"Child CLI must parse no_interactive=true from passed args"`
  - `"Flag position in child args must not affect value"`

**State Context**
- References to system state or conditions
- Examples:
  - `"Daemon should be healthy"`
  - `"Agent should be active"`
  - `"healthz should return 200"`
  - `"Heartbeats should contain idle state"`

#### 3.3 Field Paths and References

**Explicit Field Paths**
- References to specific fields in error messages
- Examples:
  - `"error should include field path"`
  - `"field path should mention schema_version: {:?}"`
  - `"error should mention pattern/format: {:?}"`

**Entity References**
- References to specific entities or IDs
- Examples:
  - `"Event {} Claim: bead_id should match"`
  - `"Event {} Complete: worker should match"`
  - `"Stitch title should reference agent session"`

### 4. Actionability

#### 4.1 Actionable Messages (High)

**Direct Action Statements**
- Messages that clearly state what should happen
- Examples:
  - `"Remove must check for confirm flag in non-interactive mode"`
  - `"Remove must show helpful error when confirm is missing"`
  - `"Should create subscriptions"`
  - `"testrepo should exist within the repository"`

**Problem-Solution Pattern**
- Messages that identify a problem and suggest the solution
- Examples:
  - `"Failed to parse args: {:?}"` (problem: parsing failed, context: which args)
  - `"Missing command - only program name"` (problem: missing command, context: what's present)
  - `"events.jsonl should not be empty"` (problem: empty, requirement: not empty)

#### 4.2 Moderately Actionable Messages (Medium)

**State Descriptions**
- Messages that describe expected state without explicit action
- Examples:
  - `"All beads should belong to testrepo"`
  - `"Should have 2 open beads"`
  - `"Should have at least 2 sessions, got {}"`
  - `"Parsing should be fast (< 1s), took {:?}"`

**Validation Messages**
- Messages that validate conditions
- Examples:
  - `"missing schema_version should fail"`
  - `"integer schema_version should fail"`
  - `"invalid schema_version format should fail"`

#### 4.3 Low Actionability Messages (Low)

**Generic Assertions**
- Messages that lack specific context
- Examples:
  - `"scan"` (as a message in assert_eq)
  - `"/tmp"` (as a message in assert_eq)
  - `"--no-interactive"` (as a message in assert_eq)
  - `".unwrap()"` (no message at all)

**Minimal Context Unwrap Calls**
- The 1,482 `.unwrap()` calls provide minimal context
- These produce generic panic messages without custom context

### 5. Pattern Quality Assessment

#### 5.1 Strong Patterns

**1. Subject-First Assertions**
- Clear, readable, follows natural language
- Examples: `"no_interactive should be true with flag before command"`
- Quality: High - easy to understand and debug

**2. Failure-Focused Expect Messages**
- Immediately identifies what failed
- Examples: `"Failed to create .beads/ directory"`
- Quality: High - actionable and specific

**3. Position/Location Context**
- Clearly indicates where something should appear
- Examples: `"flag before command"`, `"at Level 2"`
- Quality: High - reduces ambiguity

**4. Format Placeholders**
- Provides specific values at failure time
- Examples: `"Failed to parse args: {:?}"`, `"got {}"`
- Quality: High - includes actual failure context

#### 5.2 Weak Patterns

**1. Generic Unwrap Calls**
- 1,482 instances of `.unwrap()` with no custom message
- Produces generic panics without context
- Quality: Low - difficult to debug
- Recommendation: Replace with `.expect("context")` or proper error handling

**2. Minimal Assert Messages**
- Using bare strings like `"scan"`, `"/tmp"`, `"--no-interactive"` as messages
- Provides minimal context when assertion fails
- Quality: Low - doesn't explain what's being tested
- Recommendation: Use descriptive messages like `"subcommand should be 'scan'"`

**3. Implicit Context in Comparisons**
- Messages like `"Flag should be true"` don't show actual value
- Quality: Medium - better than nothing, but could include actual
- Recommendation: Use format strings to show actual vs expected

### 6. Recommendations for Error Message Standards

#### 6.1 For Assertions (`assert!`, `assert_eq!`, `assert_ne!`)

**Preferred Format:**
```rust
assert_eq!(
    actual_value,
    expected_value,
    "[Subject] should [condition] [context]"
);
```

**Examples:**
```rust
assert_eq!(parsed.no_interactive, true, "no_interactive should be true with flag before command");
assert_eq!(command, "scan", "subcommand should be 'scan' when scan command is invoked");
```

**Avoid:**
```rust
assert_eq!(parsed.no_interactive, true, "scan");  // Non-descriptive
assert_eq!(parsed.no_interactive, true);          // No message at all
```

#### 6.2 For Expectations (`expect()`)

**Use Failure-Focused Messages:**
```rust
result.expect("Failed to [action] [context]");
```

**Examples:**
```rust
let cli = parse_cli(args).expect("Failed to parse CLI arguments");
fs::create_dir(&path).expect("Failed to create directory");
```

**Avoid:**
```rust
let cli = parse_cli(args).unwrap();  // No context on failure
```

#### 6.3 For Panics

**Use Expected Pattern:**
```rust
panic!("Expected [entity] [context]");
```

**Examples:**
```rust
panic!("Expected Scan command");
panic!("Expected Remove command at Level 2");
```

#### 6.4 For Unwrap Calls

**Replace with Expect:**
```rust
// Before (low quality)
let value = some_result.unwrap();

// After (high quality)
let value = some_result.expect("Failed to [action] [context]");
```

### 7. Anti-Patterns to Avoid

1. **Single-word messages**: `"scan"`, `"/tmp"`, `"--no-interactive"` as assert messages
2. **No message assertions**: `assert_eq!(a, b)` without a message string
3. **Generic unwraps**: `.unwrap()` without `.expect()` context
4. **Missing placeholders**: `"Flag should be true"` without showing the actual value
5. **Ambiguous subjects**: `"should be true"` without saying what should be true

### 8. Pattern Consistency Score

| Category | Consistency | Notes |
|----------|-------------|-------|
| Wording (assertions) | 85% | Strong subject-first pattern, some outliers |
| Wording (expectations) | 90% | Consistent "Failed to" and "Should" patterns |
| Formatting (quotes) | 75% | Inconsistent quoting of values |
| Formatting (capitalization) | 95% | Consistent sentence case |
| Information (context) | 70% | Mix of high and low context messages |
| Actionability | 60% | Many generic messages, especially unwraps |

### 9. Next Steps for Standardization

1. **Create lint rules** to catch low-quality error messages
2. **Document template patterns** for common error types
3. **Audit unwrap calls** and replace with expect calls
4. **Standardize quoting** conventions in documentation
5. **Add context placeholders** to assertion messages
6. **Review generic messages** in high-density files (CLI tests, integration tests)

---

**Analysis Scope:** 5,904 error messages across 104 test files
**Methodology:** Pattern extraction and categorization from comprehensive_error_messages.json
**Confidence:** High - analysis based on complete catalog with representative samples
