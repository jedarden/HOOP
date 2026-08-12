# HOOP Error Message Informational and Actionability Standards

**Generated:** 2026-08-12  
**Source:** Built upon wording and formatting standards (bf-4qy5x)  
**Purpose:** Define informational content and actionability standards for error messages across HOOP  
**Task:** Define informational and actionability standards (bf-4ory0)  
**Status:** Complete

## Overview

This document defines standards for **what information error messages must include** and **how they guide users toward solutions**. It builds on the wording and formatting standards in `error_message_standards.md` and focuses on content completeness and user guidance.

### Core Principles

1. **Complete context** - Include all information needed to understand and diagnose the error
2. **Debugging efficiency** - Error messages should enable immediate diagnosis without code inspection
3. **Actionable guidance** - When appropriate, provide clear next steps or solution hints
4. **Audience awareness** - Distinguish between developer-facing and user-facing messages
5. **Balance** - Provide thoroughness without overwhelming verbosity

---

## 1. Minimum Informational Content

### 1.1 Core Information Requirements

Every error message must include at minimum:

#### A. What Failed (Required)

The specific operation, component, or validation that failed.

**✅ Correct:**
```rust
.expect("Failed to read config from ~/.hoop/config.yml")
.expect("Failed to parse bead line from events.jsonl")
.assert!(result.is_err(), "missing schema_version should fail validation")
```

**❌ Avoid:**
```rust
.expect("failed") // what failed?
.expect("error") // what error?
.assert!(false, "test") // what test?
```

#### B. Target/Subject (Required)

The specific file, field, component, or value involved in the failure.

**✅ Correct:**
```rust
"Failed to read config from ~/.hoop/config.yml"
"schema_version should be a string"
"projects.rs must exist in the repository"
```

**❌ Avoid:**
```rust
"Failed to read config" // which config?
"should be a string" // what should be a string?
"must exist" // what must exist?
```

#### C. Expected State (When Applicable)

What should have happened if the operation succeeded.

**✅ Correct:**
```rust
"Expected string, got integer"
"Expected 200 status, got 500"
"no_interactive should be true when --no-interactive is present"
```

**❌ Avoid:**
```rust
"got integer" // but what was expected?
"500 error" // what was expected?
"flag should be true" // under what condition?
```

### 1.2 Conditional Information

Include condition information when behavior differs based on state.

**✅ Correct:**
```rust
"no_interactive should be true when --no-interactive flag is present"
"parsing should succeed even when flag is at end of command"
"daemon should start when config file is valid"
```

**❌ Avoid:**
```rust
"no_interactive should be true" // missing when condition
"parsing should succeed" // missing when condition
"daemon should start" // missing prerequisite condition
```

### 1.3 Value Information

Include actual values when they differ from expectations.

**✅ Correct:**
```rust
.expect(&format!("Failed to read config from: {}", path))
.assert_eq!(value, expected, "field should match: expected {}, got {}", expected, value)
```

**❌ Avoid:**
```rust
.expect("Failed to read config") // which path?
.assert_eq!(value, expected) // no context on mismatch
```

---

## 2. When to Include Context

### 2.1 File and Location Context

**Include file context when:**
- The error originates from file I/O operations
- Multiple files could be the source
- The path is user-configurable
- The file is critical for operation

**✅ Include:**
```rust
.expect("Failed to read config from ~/.hoop/config.yml")
.expect("Failed to parse events.jsonl at line 15")
.expect("Failed to create .beads/ directory")
```

**❌ Omit (when single hardcoded path):**
```rust
.expect("Failed to read config") // OK if only one config file in codebase
.expect("Failed to parse bead line") // OK if parsing is single-purpose
```

### 2.2 Function and Operation Context

**Include function context when:**
- Multiple operations could produce the same error
- The function name aids diagnosis
- The operation is non-obvious from context

**✅ Include:**
```rust
.expect("ConfigParser::parse() failed: invalid schema_version")
.expect("BeadDeserialization::from_line() failed: missing claimed_at")
```

**❌ Omit (when obvious):**
```rust
.expect("Failed to read config") // obvious from code
.assert!(value, "validation failed") // obvious from assertion type
```

### 2.3 Field and Property Context

**Always include field context for validation errors.**

**✅ Include:**
```rust
"schema_version should be a string, not an integer"
"projects.rs path must be valid"
"claimed_at timestamp must be parsable"
```

**❌ Avoid:**
```rust
"should be a string" // which field?
"must be valid" // which property?
"must be parsable" // which field?
```

### 2.4 Component and Subsystem Context

**Include component context when:**
- Multiple components could produce the error
- The component identity aids in routing the fix
- The error is component-specific

**✅ Include:**
```rust
"Daemon should start without errors when config is valid"
"CLI parser should recognize --no-interactive flag"
"MCP server should reject create_stitch calls in observer mode"
```

**❌ Omit (when global context):**
```rust
"should start without errors" // OK if test name makes component clear
"should recognize flag" // OK if scoped to CLI tests
```

---

## 3. When and How to Provide Suggestions for Fixes

### 3.1 When to Provide Fix Suggestions

**Provide suggestions when:**
- The fix is straightforward and non-ambiguous
- The error has a common, well-known solution
- The user can directly act on the suggestion
- The suggestion doesn't require deep system knowledge

**✅ Provide suggestions:**
```rust
// In user-facing CLI errors
"Failed to read config.yml: file not found. Run 'hoop init' to create default config"

// In validation errors with clear fixes
"schema_version must be a string. In config.yml, change: schema_version: 1 → schema_version: '1'"

// In setup errors
"projects.rs not found. Ensure you're in a valid HOOP workspace root"
```

**❌ Don't provide suggestions (when):**
- The fix is complex or multi-step
- Multiple potential causes exist
- The suggestion would be speculative
- The error requires system-level diagnosis

### 3.2 How to Structure Suggestions

#### Pattern A: Direct Fix (User-Facing)

**Structure:** `<error>. <fix>`

```rust
// ✅ Correct
"Failed to read config.yml: permission denied. Check file permissions with: ls -la ~/.hoop/config.yml"
"Invalid bead ID format. Bead IDs must start with 'bf-' prefix"
```

#### Pattern B: Diagnostic Hint (Developer-Facing)

**Structure:** `<error>. Hint: <diagnostic_tip>`

```rust
// ✅ Correct
"Failed to deserialize bead. Hint: Check that bead line format matches: <id>|<title>|<status>"
"Config validation failed. Hint: Run with --debug to see full validation errors"
```

#### Pattern C: Reference Documentation (Complex Issues)

**Structure:** `<error>. See: <documentation_reference>`

```rust
// ✅ Correct
"Config schema_version 2 is not supported by this daemon version. See docs/config_migration.md"
"Reflection ledger corrupted. See docs/operations.md#disaster-recovery for recovery steps"
```

### 3.3 Suggestion Quality Standards

**Good suggestions:**
- Are actionable (user can do something immediately)
- Are specific (name exact commands or changes)
- Are safe (won't make things worse)
- Are accurate (actually solve the problem)

**✅ Good:**
```rust
"Failed to open config.yml: permission denied. Run: chmod 600 ~/.hoop/config.yml"
"Invalid schema_version: expected '1', got 2. Downgrade config or update daemon"
```

**❌ Poor:**
```rust
"Failed to open config.yml. Check permissions" // what permissions?
"Invalid schema_version. Fix config" // how to fix?
"Config error. Try again" // not actionable
```

---

## 4. Structuring Actionable Error Messages

### 4.1 Actionable Message Structure

**Standard structure:** `[Problem] + [Context] + [Consequence] + [Action]`

Not all components are required; include what's relevant.

#### Full Structure Example

```rust
// Problem + Context + Consequence + Action
"Config file ~/.hoop/config.yml is missing (problem). Daemon cannot start without config (consequence). Run 'hoop init' to create default config (action)"
```

#### Simplified Structures

```rust
// Problem + Context
"Failed to read config from ~/.hoop/config.yml: file not found"

// Problem + Context + Action
"Invalid schema_version: expected string, got integer. Change schema_version: 1 to schema_version: '1' in config.yml"

// Problem + Consequence
"projects.rs not found in repository. Cannot proceed without workspace metadata"
```

### 4.2 Actionability Levels

#### Level 1: Self-Explanatory (Developer Internal)

Errors where the message itself is sufficient for diagnosis.

```rust
// ✅ Level 1 - sufficient
"Failed to parse bead line: invalid claimed_at timestamp format"
"schema_version should be a string, not an integer"
```

#### Level 2: Diagnostic Guidance (Needs Investigation)

Errors that guide where to look but don't provide the fix.

```rust
// ✅ Level 2 - guides investigation
"Failed to read config from ~/.hoop/config.yml. Check file permissions and format"
"Config validation failed. Run with --debug flag for detailed errors"
```

#### Level 3: Direct Action (User-Facing)

Errors that provide or imply the specific fix.

```rust
// ✅ Level 3 - actionable
"Config file not found. Run 'hoop init' to create default configuration"
"Invalid bead ID format. Bead IDs must start with 'bf-' prefix. Got: {}"
```

### 4.3 Audience-Based Actionability

**Developer-facing errors (internal diagnostics):**
```rust
// ✅ Developer-facing - technical depth
"Failed to deserialize bead line at offset 12345: missing claimed_at field. Line content: {:?}"
"Config validation failed at field 'schema_version': type mismatch in YAML parsing"
```

**User-facing errors (CLI output):**
```rust
// ✅ User-facing - simplified, actionable
"Config file is invalid. Run 'hoop config validate' for details"
"Failed to create bead: workspace directory not found. Check that you're in a valid workspace"
```

---

## 5. When Actionability is Appropriate vs Purely Informational

### 5.1 Use Actionable Messages When

**✅ Use actionability for:**
- User-facing CLI errors
- Setup and initialization errors
- Configuration validation errors
- File permission errors
- Missing dependency errors
- Invalid input errors (with clear fix)

**Examples:**
```rust
"Config file not found. Run 'hoop init' to create default configuration"
"Failed to open config.yml: permission denied. Run: chmod 600 ~/.hoop/config.yml"
"Invalid bead ID: 'invalid-id'. Bead IDs must start with 'bf-'"
```

### 5.2 Use Purely Informational Messages When

**✅ Use informational-only for:**
- Internal diagnostics and debugging
- Test assertion messages
- Invariant violations (developer-only)
- Complex errors requiring investigation
- Errors with multiple potential causes

**Examples:**
```rust
// Internal diagnostics
"Failed to deserialize bead line: missing claimed_at field. Line: {:?}"

// Test assertions
"no_interactive should be true when --no-interactive flag is present"

// Invariant violations
"projects.rs must exist in the repository before daemon starts"

// Complex errors
"Config validation failed: multiple errors. Run with --debug for details"
```

### 5.3 Mixed Approach (Informational + Diagnostic Hint)

**When errors are complex but have diagnostic value:**

```rust
// ✅ Informational error with diagnostic hint
"Failed to parse bead line at offset 12345. Hint: Check that line format matches: <id>|<title>|<status>|...|"

// ✅ Informational with debug guidance
"Config validation failed. Hint: Run with --debug flag to see detailed validation errors"

// ✅ Informational with reference
"Reflection ledger consistency check failed. See docs/operations.md#reflection-ledger for recovery procedures"
```

---

## 6. Complete Examples

### Example 1: File I/O Error (User-Facing)

**❌ Poor:**
```rust
let config = std::fs::read_to_string(path).unwrap();
```

**✅ Good:**
```rust
let config = std::fs::read_to_string(path)
    .expect(&format!("Failed to read config from: {}. Run 'hoop init' if config doesn't exist", path));
```

**Improvements:**
- What failed: read config
- Target: specific path
- Action: initialization hint
- User-facing language

### Example 2: Validation Error (Developer-Facing)

**❌ Poor:**
```rust
assert!(result.is_err());
```

**✅ Good:**
```rust
assert!(result.is_err(), 
    "missing schema_version should fail validation. Config requires schema_version field in root");
```

**Improvements:**
- What failed: validation
- Expected: error on missing field
- Context: why validation should fail
- Developer-facing depth

### Example 3: Type Mismatch (Internal Diagnostic)

**❌ Poor:**
```rust
panic!("expected string, got integer");
```

**✅ Good:**
```rust
panic!(
    "Config schema_version type mismatch: expected string, got integer. \
    In config.yml, change schema_version: 1 to schema_version: '1'"
);
```

**Improvements:**
- What failed: type check
- Expected vs actual: explicit
- Context: config file location
- Action: how to fix
- Shows both diagnostic and actionability

### Example 4: Invariant Violation (Internal)

**❌ Poor:**
```rust
assert!(exists, "file must exist");
```

**✅ Good:**
```rust
assert!(exists, 
    "projects.rs must exist in the repository. Daemon requires workspace metadata to initialize. \
    Ensure you're in a valid HOOP workspace root");
```

**Improvements:**
- What failed: invariant check
- Why it matters: daemon requires it
- Context: workspace root requirement
- Actionable: verify workspace location

### Example 5: Complex Error with Hint

**❌ Poor:**
```rust
.expect("Failed to deserialize bead");
```

**✅ Good:**
```rust
.expect(
    "Failed to deserialize bead line. \
    Hint: Bead line format is: <id>|<title>|<status>|...| \
    Check that line contains all required fields"
);
```

**Improvements:**
- What failed: deserialization
- Context: bead line format
- Diagnostic hint: what to check
- Developer-facing (not specific fix, just guidance)

---

## 7. Anti-Patterns to Avoid

### 7.1 Over-Actionable Messages

**❌ Avoid speculating on fixes when the cause is unclear:**
```rust
// ❌ Too speculative
"Failed to read config. Maybe file permissions are wrong? Try reinstalling"

// ✅ Better - diagnostic
"Failed to read config from: {}. Check file exists and is readable", path
```

### 7.2 Over-Verbose Messages

**❌ Avoid overwhelming detail:**
```rust
// ❌ Too verbose
"Failed to read config file located at path /home/user/.hoop/config.yml with error code 2 indicating \
file not found which means the file does not exist at the specified location"

// ✅ Better - concise
"Failed to read config from ~/.hoop/config.yml: file not found. Run 'hoop init' to create config"
```

### 7.3 Vague Action Suggestions

**❌ Avoid non-specific suggestions:**
```rust
// ❌ Vague
"Config error. Check the config file"
"Failed to parse. Fix the error"

// ✅ Better - specific
"Config validation failed at field 'schema_version': expected string, got integer"
"Failed to parse bead line: missing claimed_at field at line 15"
```

### 7.4 Missing Critical Context

**❌ Never omit context that's needed for diagnosis:**
```rust
// ❌ Missing target
"Failed to read config" // which config?

// ❌ Missing expected value
"got integer" // but what was expected?

// ❌ Missing field
"validation failed" // what failed validation?

// ✅ Always include
"Failed to read config from ~/.hoop/config.yml"
"Expected string, got integer"
"schema_version validation failed: must be string"
```

---

## 8. Standards Summary

### 8.1 Minimum Content Requirements

Every error message must include:
- **What failed** - The operation or validation
- **Target/subject** - The specific file, field, or component
- **Expected state** (when applicable) - What should have happened

### 8.2 Context Inclusion Rules

Include context when:
- Multiple sources could produce the error (files, components)
- The context aids in routing the fix
- The error is non-obvious from code inspection
- The value is user-configurable or variable

### 8.3 Actionability Guidelines

**Use actionability when:**
- Fix is straightforward and non-ambiguous
- Error is user-facing (CLI, initialization)
- Solution is a common, well-known pattern
- User can directly act on suggestion

**Use informational-only when:**
- Error is internal (diagnostics, tests)
- Fix is complex or multi-step
- Multiple potential causes exist
- Error requires system-level investigation

**Use diagnostic hints when:**
- Error is complex but has diagnostic value
- Suggestion guides investigation without prescribing fix
- Documentation reference provides complete solution

### 8.4 Quality Checklist

Before committing new error messages, verify:

**Content completeness:**
- [ ] Message identifies what failed
- [ ] Message names the target/subject
- [ ] Message includes expected state when relevant
- [ ] Message includes actual values when they differ
- [ ] Message includes condition when behavior is conditional

**Context appropriateness:**
- [ ] File context included when relevant
- [ ] Component context included when multiple components could fail
- [ ] Field context included for validation errors
- [ ] Function context included when operation is non-obvious

**Actionability appropriateness:**
- [ ] Action suggestions are specific and accurate
- [ ] Action suggestions are safe to follow
- [ ] Diagnostic hints guide investigation effectively
- [ ] No speculative or vague suggestions
- [ ] Audience-appropriate depth (user vs developer)

**Format compliance (from error_message_standards.md):**
- [ ] Follows "should"/"Failed to"/"must" patterns
- [ ] Preserves original case for identifiers
- [ ] Uses `{:?}` for debug, `{}` for display
- [ ] No trailing period
- [ ] Placeholders at end of message

---

## 9. Examples by Category

### 9.1 File Operation Errors

**✅ Good:**
```rust
"Failed to read config from ~/.hoop/config.yml: file not found. Run 'hoop init' to create default config"
"Failed to create .beads/ directory: permission denied. Check parent directory permissions"
"Failed to parse events.jsonl: line 15 contains invalid bead format"
```

### 9.2 Validation Errors

**✅ Good:**
```rust
"schema_version must be a string, not an integer. Change schema_version: 1 to schema_version: '1'"
"projects.rs path must be valid and absolute. Got: {}", path
"Bead ID must start with 'bf-' prefix. Got: {}", bead_id
```

### 9.3 Setup/Initialization Errors

**✅ Good:**
```rust
"Config file not found at ~/.hoop/config.yml. Run 'hoop init' to create default configuration"
"projects.rs not found in repository. Ensure you're in a valid HOOP workspace root"
"Daemon cannot start: missing required dependencies. Run 'cargo build' to compile"
```

### 9.4 Internal Diagnostic Errors

**✅ Good:**
```rust
"Failed to deserialize bead line at offset {}: missing claimed_at field. Line content: {:?}", offset, line
"Config validation failed at field '{}': type mismatch in YAML parsing. Expected: {}, Got: {}", field, expected, actual
"Bead state projection inconsistent: expected state {}, found state {}", expected, actual
```

### 9.5 Test Assertion Messages

**✅ Good:**
```rust
"no_interactive should be true when --no-interactive flag is present"
"missing schema_version should fail validation"
"CLI should parse --no-interactive flag correctly even when it appears at end of command"
"Both positions in bead line must yield the same claimed_at value"
```

---

## 10. Integration with Wording and Formatting Standards

This document complements the **wording and formatting standards** in `error_message_standards.md`:

- **Wording/formatting:** How to phrase and punctuate error messages
- **Informational/actionability:** What content to include and how to guide users

Use both documents together:

1. Apply **wording patterns** ("should", "Failed to", "must") from formatting standards
2. Apply **content requirements** (what, target, expected) from this document
3. Apply **actionability guidelines** (when to suggest fixes) from this document
4. Apply **formatting rules** (no periods, placeholder placement) from formatting standards

---

## Appendix: Quick Reference

### Informational Content Requirements

| Component | Required? | When Required | Example |
|-----------|-----------|---------------|---------|
| What failed | ✅ Always | - | Failed to read config |
| Target/subject | ✅ Always | - | from ~/.hoop/config.yml |
| Expected state | ✅ When applicable | Value comparisons | Expected string, got integer |
| Actual value | ✅ When differs from expected | Value mismatches | Expected 200, got 500 |
| Condition | ✅ When behavior is conditional | Conditional behavior | when --no-interactive is present |
| File location | ⚡️ When relevant | Multiple possible files | at line 15 in events.jsonl |
| Component | ⚡️ When relevant | Multiple components | CLI parser should recognize |

### Actionability Decision Tree

```
Is the error user-facing (CLI, setup)?
├─ Yes → Is the fix straightforward?
│  ├─ Yes → Provide direct action
│  │         Example: "Run 'hoop init' to create config"
│  └─ No → Provide diagnostic hint
│            Example: "Run with --debug for details"
└─ No (internal) → Is diagnostic value clear?
   ├─ Yes → Provide diagnostic hint
   │         Example: "Hint: Check line format matches..."
   └─ No → Informational only
              Example: "Failed to deserialize bead line"
```

### Message Structure Templates

| Purpose | Template | Example |
|---------|----------|---------|
| Self-explanatory | `[What] [Target] [Context]` | Failed to read config from ~/.hoop/config.yml |
| With action | `[Error]. [Action]` | Config not found. Run 'hoop init' |
| With hint | `[Error]. Hint: [Diagnostic]` | Parse failed. Hint: Check line format |
| With reference | `[Error]. See: [Doc]` | Ledger corrupted. See docs/operations.md |
| Full structure | `[Problem] + [Context] + [Consequence] + [Action]` | Config missing. Daemon cannot start. Run 'hoop init' |

---

**Document Status:** Complete  
**Related Documents:** 
- `error_message_standards.md` (wording and formatting)
- `error_messages_catalog.md` (existing error patterns)

**Next Steps:** Apply these informational and actionability standards in error message improvement work across HOOP codebase.
