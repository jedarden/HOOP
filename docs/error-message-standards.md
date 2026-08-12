# HOOP Error Message Standards — Complete Reference

**Generated:** 2026-08-12  
**Purpose:** Comprehensive reference for error message standards across HOOP  
**Bead:** bf-3w4k2 (consolidated standards)  
**Related:** bf-4qy5x (wording/formatting), bf-4ory0 (informational/actionability)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Quick Reference](#2-quick-reference)
3. [Core Principles](#3-core-principles)
4. [Sentence Structure Standards](#4-sentence-structure-standards)
5. [Informational Content Requirements](#5-informational-content-requirements)
6. [Context Inclusion Guidelines](#6-context-inclusion-guidelines)
7. [Actionability and Suggestions](#7-actionability-and-suggestions)
8. [Format Rules](#8-format-rules)
9. [Audience Guidelines](#9-audience-guidelines)
10. [Complete Examples](#10-complete-examples)
11. [Anti-Patterns to Avoid](#11-anti-patterns-to-avoid)
12. [Validation Checklist](#12-validation-checklist)
13. [Migration Path](#13-migration-path)

---

## 1. Overview

This document consolidates all HOOP error message standards into a single comprehensive reference. It combines:

- **Wording and formatting standards** — How to phrase and format error messages
- **Informational content standards** — What information error messages must include
- **Actionability standards** — When and how to provide fix suggestions

Error messages in HOOP serve two primary purposes:

1. **Debugging efficiency** — Enable immediate diagnosis without code inspection
2. **User guidance** — Guide users toward solutions when appropriate

---

## 2. Quick Reference

### 2.1 Minimum Content Formula

```
What failed + Target + Expected state [+ Actual state if different] [+ Context if relevant] [+ Action if appropriate]
```

**Example:**
```
"Failed to read ~/.hoop/config.yml: file not found. Run 'hoop init' to create it"
 What      target                   cause               suggestion
```

### 2.2 Standard Pattern Templates

| Purpose | Template | Example |
|---------|----------|---------|
| Expected behavior | `<subject> should <state> [when <condition>]` | `no_interactive should be true when --no-interactive is present` |
| Operation failure | `Failed to <action> <target> [because <reason>]` | `Failed to read config: file not found` |
| Invariant | `<subject> must <condition>` | `projects.rs must exist in repository` |
| Value comparison | `Expected <expected>, got <actual>` | `Expected string, got integer` |
| Validation test | `<action> should <result>` | `missing schema_version should fail validation` |

### 2.3 Actionability Decision Tree

```
Is the error user-facing (CLI, setup)?
├─ Yes → Is the fix straightforward and safe?
│  ├─ Yes → Provide direct action
│  │         Example: "Run 'hoop init' to create config"
│  └─ No → Provide diagnostic hint
│            Example: "Run with --debug for details"
└─ No (internal/developer) → Is diagnostic value clear?
   ├─ Yes → Provide diagnostic hint
   │         Example: "Hint: Check line format matches..."
   └─ No → Informational only
              Example: "Failed to deserialize bead line"
```

### 2.4 Format Rules at a Glance

| Rule | Correct ✅ | Incorrect ❌ |
|------|-----------|-------------|
| No trailing periods | `Failed to read config` | `Failed to read config.` |
| No unnecessary quotes | `should be true` | `should be 'true'` |
| Capitalize first word | `Failed to read config` | `failed to read config` |
| Preserve original case | `--no-interactive` | `--no_interactive` |
| Placeholders at end | `Failed: {}` | `Failed {} to read` |
| Debug vs display | `{:?}` for dev, `{}` for user | Opposite |

### 2.5 Context Inclusion Quick Guide

| Context Type | Include When | Example |
|--------------|--------------|---------|
| **File path** | I/O operations, user files, config | `~/.hoop/config.yml` |
| **Function name** | Public API, ambiguous origin | `br_create() failed` |
| **Line/column** | Parsing errors | `line 15, column 3` |
| **Field name** | Validation errors | `schema_version must be string` |
| **Component** | Multiple components could fail | `Daemon should start` |
| **System state** | Resource errors | `(3/5 slots in use)` |

---

## 3. Core Principles

1. **Clarity over brevity** — Prefer descriptive messages that explain what happened and why
2. **Complete context** — Include all information needed to understand and diagnose the error
3. **Actionability when appropriate** — Provide clear next steps or solution hints when safe and obvious
4. **Audience awareness** — Distinguish between developer-facing and user-facing messages
5. **Consistency** — Follow established patterns across the codebase
6. **Context preservation** — Maintain original casing for CLI flags, filenames, and identifiers

---

## 4. Sentence Structure Standards

### 4.1 Pattern A: "Should" Pattern (Primary)

**Structure:** `<subject> should <expected_state> [when <condition>]`

**Usage:**
- Positive assertions about expected behavior
- State validation in tests
- Component behavior verification

**Examples:**
```rust
// ✅ Correct
"no_interactive should be true when --no-interactive flag is present"
"schema_version should be a string"
"healthz endpoint should return 200 status"

// ❌ Avoid
"no_interactive must be true" // "must" is for invariants
"flag true" // too minimal
"should be true" // what should be true?
```

**Rationale:** Most common pattern in codebase (25.4%), declarative, easy to understand.

### 4.2 Pattern B: "Failed to" Pattern (Operations)

**Structure:** `Failed to <action> <target> [because <reason>]`

**Usage:**
- File I/O operations
- Setup/teardown failures
- External system interactions
- Any operation that doesn't complete

**Examples:**
```rust
// ✅ Correct
.expect("Failed to read config from ~/.hoop/config.yml")
.expect("Failed to create .beads/ directory")
.expect("Failed to parse events.jsonl")

// ❌ Avoid
.expect("failed reading") // what failed reading?
.expect("Failed") // failed to do what?
```

**Rationale:** Widely used (31.7%), clearly identifies what went wrong and target.

### 4.3 Pattern C: "Must" Pattern (Invariants)

**Structure:** `<subject> must <condition>`

**Usage:**
- Critical invariants that must always hold
- Setup requirements that cannot be bypassed
- Security/safety constraints

**Examples:**
```rust
// ✅ Correct
"projects.rs must exist in the repository"
"schema_version must be a string, not an integer"
"config file must be readable before daemon starts"

// ❌ Avoid
"projects.rs should exist" // "should" is for preferences
```

**Rationale:** "Must" expresses stronger assertions; reserve for invariants.

### 4.4 Conditional Phrasing

**Standard:** Use `when <condition>` for contextual information

```rust
// ✅ Correct
"no_interactive should be true when --no-interactive flag is present"
"parsing should succeed even when flag is at end of command"

// ❌ Avoid
"no_interactive should be true with flag" // what flag?
"flag should be true in this case" // what case?
```

### 4.5 Action + Outcome Pattern

**Structure:** `<action> should <result>`

**Usage:**
- Validation testing
- Error path verification
- Command behavior testing

```rust
// ✅ Correct
"missing schema_version should fail validation"
"invalid schema_version format should return error"
"error should include field path"
```

---

## 5. Informational Content Requirements

### 5.1 Core Information Requirements (Always Required)

Every error message must include at minimum:

#### A. What Failed (Required)

The specific operation, component, or validation that failed.

```rust
// ✅ Correct
.expect("Failed to read config from ~/.hoop/config.yml")
.expect("Failed to parse bead line from events.jsonl")
.assert!(result.is_err(), "missing schema_version should fail validation")

// ❌ Avoid
.expect("failed") // what failed?
.expect("error") // what error?
.assert!(false, "test") // what test?
```

#### B. Target/Subject (Required)

The specific file, field, component, or value involved.

```rust
// ✅ Correct
"Failed to read config from ~/.hoop/config.yml"
"schema_version should be a string"
"projects.rs must exist in the repository"

// ❌ Avoid
"Failed to read config" // which config?
"should be a string" // what should be?
"must exist" // what must exist?
```

#### C. Expected State (When Applicable)

What should have happened if the operation succeeded.

```rust
// ✅ Correct
"Expected string, got integer"
"Expected 200 status, got 500"
"no_interactive should be true when --no-interactive is present"

// ❌ Avoid
"got integer" // but what was expected?
"500 error" // what was expected?
"flag should be true" // under what condition?
```

### 5.2 Conditional Information (Include When Relevant)

#### Condition Information

Include when behavior differs based on state.

```rust
// ✅ Correct
"no_interactive should be true when --no-interactive flag is present"
"parsing should succeed even when flag is at end of command"
"daemon should start when config file is valid"

// ❌ Avoid
"no_interactive should be true" // missing when condition
"parsing should succeed" // missing when condition
```

#### Value Information

Include actual values when they differ from expectations.

```rust
// ✅ Correct
.expect(&format!("Failed to read config from: {}", path))
.assert_eq!(value, expected, "field should match: expected {}, got {}", expected, value)

// ❌ Avoid
.expect("Failed to read config") // which path?
.assert_eq!(value, expected) // no context on mismatch
```

---

## 6. Context Inclusion Guidelines

### 6.1 File and Location Context

**Include file context when:**
- The error originates from file I/O operations
- Multiple files could be the source
- The path is user-configurable
- The file is critical for operation

```rust
// ✅ Include
.expect("Failed to read config from ~/.hoop/config.yml")
.expect("Failed to parse events.jsonl at line 15")
.expect("Failed to create .beads/ directory")

// ❌ Omit (when single hardcoded path)
.expect("Failed to read config") // OK if only one config
.expect("Failed to parse bead line") // OK if parsing is single-purpose
```

### 6.2 Function and Operation Context

**Include function context when:**
- Multiple operations could produce the same error
- The function name aids diagnosis
- The operation is non-obvious from context

```rust
// ✅ Include
.expect("ConfigParser::parse() failed: invalid schema_version")
.expect("BeadDeserialization::from_line() failed: missing claimed_at")

// ❌ Omit (when obvious)
.expect("Failed to read config") // obvious from code
.assert!(value, "validation failed") // obvious from assertion type
```

### 6.3 Field and Property Context

**Always include field context for validation errors.**

```rust
// ✅ Include
"schema_version should be a string, not an integer"
"projects.rs path must be valid"
"claimed_at timestamp must be parsable"

// ❌ Avoid
"should be a string" // which field?
"must be valid" // which property?
```

### 6.4 Component and Subsystem Context

**Include component context when:**
- Multiple components could produce the error
- The component identity aids in routing the fix
- The error is component-specific

```rust
// ✅ Include
"Daemon should start without errors when config is valid"
"CLI parser should recognize --no-interactive flag"
"MCP server should reject create_stitch calls in observer mode"

// ❌ Omit (when global context)
"should start without errors" // OK if test name makes component clear
"should recognize flag" // OK if scoped to CLI tests
```

---

## 7. Actionability and Suggestions

### 7.1 When to Provide Fix Suggestions

**Provide suggestions when:**
- The fix is straightforward and non-ambiguous
- The error has a common, well-known solution
- The user can directly act on the suggestion
- The suggestion doesn't require deep system knowledge

```rust
// ✅ Provide suggestions
"Failed to read config.yml: file not found. Run 'hoop init' to create default config"
"schema_version must be a string. In config.yml, change: schema_version: 1 → schema_version: '1'"
"projects.rs not found. Ensure you're in a valid HOOP workspace root"

// ❌ Don't provide suggestions when:
// - The fix is complex or multi-step
// - Multiple potential causes exist
// - The suggestion would be speculative
// - The error requires system-level diagnosis
```

### 7.2 How to Structure Suggestions

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

### 7.3 Actionable Message Structure

**Standard structure:** `[Problem] + [Context] + [Consequence] + [Action]`

Not all components are required; include what's relevant.

```rust
// Full structure
"Config file ~/.hoop/config.yml is missing (problem). Daemon cannot start without config (consequence). Run 'hoop init' to create default config (action)"

// Simplified structures
"Failed to read config from ~/.hoop/config.yml: file not found" // Problem + Context
"Invalid schema_version: expected string, got integer. Change schema_version: 1 to schema_version: '1'" // Problem + Context + Action
"projects.rs not found in repository. Cannot proceed without workspace metadata" // Problem + Consequence
```

### 7.4 Audience-Based Actionability

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

## 8. Format Rules

### 8.1 Punctuation Standards

#### Periods

**Standard:** Do NOT end error messages with periods

```rust
// ✅ Correct
.expect("Failed to read config")
.assert_eq!(value, true, "flag should be true")

// ❌ Avoid
.expect("Failed to read config.") // trailing period
.assert_eq!(value, true, "flag should be true.") // trailing period
```

**Rationale:** Error messages are fragments, not sentences.

#### Quotes Around Values

**Standard:** Do NOT use quotes around simple values unless necessary for clarity

**When to omit quotes:**
- Boolean values: `true`, `false`
- Commands: `scan`, `projects`, `remove`
- CLI flags: `--no-interactive`, `-y`
- Simple strings: `schema_version`, `.beads/`

**When to use quotes:**
- Values containing spaces: `"error message"`
- Ambiguous strings: `"true"` (as a string, not boolean)
- User-facing text: `"Please try again"`

```rust
// ✅ Correct - no quotes needed
"no_interactive should be true"
"command should be scan"

// ✅ Correct - quotes for clarity
"error message should contain 'invalid'"

// ❌ Avoid - unnecessary quotes
"no_interactive should be 'true'"
"command should be 'scan'"
```

#### Commas

**Standard:** Use commas before clauses and in lists

```rust
// ✅ Correct
"Failed to read config, but daemon should continue"
"file, path, and line should all be present"

// ❌ Avoid
"Failed to read config but daemon should continue" // missing comma
```

#### Colons

**Standard:** Use colons before format placeholders or explanations

```rust
// ✅ Correct
"field path should mention: schema_version"
"error should mention pattern: {:?}"
"Failed to read: {}"

// ❌ Avoid
"error should mention pattern {:?}" // colon missing
```

### 8.2 Capitalization Conventions

#### Component Names

**Standard:** Use consistent casing

- Acronyms/initialisms: Uppercase (`CLI`, `API`, `HTTP`, `JSON`)
- Components: Title case (`Daemon`, `Handler`, `Manager`)

```rust
// ✅ Correct
"CLI should parse flag correctly"
"Daemon should start without errors"
"API should return 200 status"
```

#### Preserve Original Case

**Standard:** Preserve original casing for system identifiers

```rust
// ✅ Correct - preserve original
"projects.rs must exist"
"--no-interactive flag should be true"
"scan command should require confirmation"
".beads/ directory should be created"

// ❌ Avoid - changing original case
"Projects.rs must exist" // filename case changed
"--no_interactive flag" // flag format changed
"Scan command" // command case changed
```

#### First Word Capitalization

**Standard:** Capitalize the first word of the message

```rust
// ✅ Correct
"Failed to read config"
"CLI should parse flag"
"schema_version should be a string"

// ❌ Avoid
"failed to read config" // lowercase first word
```

### 8.3 Actual vs Expected Value Presentation

**Structure:** `Expected <expected>, got <actual>` or `expected: <expected>, actual: <actual>`

```rust
// ✅ Correct
"Expected string, got integer"
"expected: true, actual: false"
"Expected 200 status, got 500"

// ✅ With placeholders
"Expected {}, got {:?}", expected, actual

// ❌ Avoid
"true vs false" // which is expected?
"integer but got string" // awkward
```

### 8.4 Format Placeholder Usage

**Standard:** Use `{:?}` for debugging, `{}` for user-facing

```rust
// ✅ Debug formatting (developer-facing)
"Failed to parse config: {:?}", error
"field path should mention: {:?}", actual_path

// ✅ Display formatting (user-facing)
"Failed to open file: {}", filename
"Expected command: {}, got: {}", expected_cmd, actual_cmd

// ❌ Avoid
"Failed to read: {:?}" // user doesn't need debug format
"error message: {}" // developer needs more context
```

### 8.5 Placeholder Placement

**Standard:** Place format placeholders at the end of messages

```rust
// ✅ Correct
"Failed to read config: {}", filename
"field path should mention: {:?}", path
"Expected {}, got {}", expected, actual

// ❌ Avoid
"Failed {} to read config" // awkward
"field path {} should mention: {:?}", path, value // confusing
```

---

## 9. Audience Guidelines

### 9.1 Developer-Facing Messages

**Characteristics:**
- Debug formatting (`{:?}`)
- Code locations and stack traces
- Full technical context
- Complex type information

**Use for:**
- Test assertion messages
- Internal diagnostics
- Development-time errors
- Invariant violations

```rust
// ✅ Developer-facing
"Failed to deserialize bead line at offset 12345: missing claimed_at field. Line content: {:?}"
"Config validation failed at field 'schema_version': type mismatch in YAML parsing"
"Bead state projection inconsistent: expected state {:?}, found state {:?}"
```

### 9.2 User-Facing Messages

**Characteristics:**
- Display formatting (`{}`)
- User-visible entities
- Clear, non-technical language
- Actionable suggestions when possible

**Use for:**
- CLI error messages
- API error responses
- Setup/initialization errors
- Configuration validation

```rust
// ✅ User-facing
"Config file is invalid. Run 'hoop config validate' for details"
"Failed to create bead: workspace directory not found. Check that you're in a valid workspace"
"Invalid bead ID format. Bead IDs must start with 'bf-' prefix"
```

### 9.3 Mixed Audience (Diagnostic + Actionable)

When errors span audiences, provide both:

```rust
// ✅ Mixed - technical info + user action
"Failed to read config from ~/.hoop/config.yml: permission denied. \
Run: ls -la ~/.hoop/config.yml to check permissions"

"schema_version type mismatch: expected string, got integer. \
In config.yml, change schema_version: 1 to schema_version: '1'"
```

---

## 10. Complete Examples

### 10.1 File I/O Error (User-Facing)

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

### 10.2 Validation Error (Developer-Facing)

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

### 10.3 Type Mismatch (Internal Diagnostic)

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

### 10.4 Invariant Violation (Internal)

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

### 10.5 Complex Error with Hint

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
- Developer-facing (guidance, not specific fix)

---

## 11. Anti-Patterns to Avoid

### 11.1 Cryptic Messages

**❌ Avoid:**
```rust
"test failed"
"error occurred"
"invalid"
```

**✅ Instead:**
```rust
"config validation should fail with missing schema_version"
"Failed to parse config.yml: line 15, column 3"
"invalid schema_version: expected string, got integer"
```

### 11.2 Over-Vague References

**❌ Avoid:**
```rust
"flag should be true"
"file must exist"
"value should be correct"
```

**✅ Instead:**
```rust
"no_interactive flag should be true when --no-interactive is present"
"projects.rs must exist in the repository"
"schema_version should be a string value"
```

### 11.3 Missing Context

**❌ Avoid:**
```rust
.unwrap()
.expect("failed")
.assert_eq!(value, expected)
```

**✅ Instead:**
```rust
.expect("Failed to read config from ~/.hoop/config.yml")
.expect(&format!("Failed to create directory: {}", path))
.assert_eq!(value, expected, "field should match expected value: {:?}", expected)
```

### 11.4 Inconsistent Terminology

**❌ Avoid:**
```rust
"cli should parse" // sometimes CLI, sometimes cli
"testrepo must exist" // sometimes testRepo
"--no_interactive" // sometimes --no-interactive
```

**✅ Instead:**
```rust
"CLI should parse" // always uppercase
"testrepo must exist" // consistent casing
"--no-interactive" // preserve original
```

### 11.5 Over-Actionable Messages

**❌ Avoid speculating on fixes when the cause is unclear:**
```rust
// ❌ Too speculative
"Failed to read config. Maybe file permissions are wrong? Try reinstalling"

// ✅ Better - diagnostic
"Failed to read config from: {}. Check file exists and is readable", path
```

### 11.6 Over-Verbose Messages

**❌ Avoid overwhelming detail:**
```rust
// ❌ Too verbose
"Failed to read config file located at path /home/user/.hoop/config.yml with error code 2 indicating \
file not found which means the file does not exist at the specified location"

// ✅ Better - concise
"Failed to read config from ~/.hoop/config.yml: file not found. Run 'hoop init' to create config"
```

### 11.7 Vague Action Suggestions

**❌ Avoid non-specific suggestions:**
```rust
// ❌ Vague
"Config error. Check the config file"
"Failed to parse. Fix the error"

// ✅ Better - specific
"Config validation failed at field 'schema_version': expected string, got integer"
"Failed to parse bead line: missing claimed_at field at line 15"
```

### 11.8 Missing Critical Context

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

## 12. Validation Checklist

Use this checklist before committing new error messages:

### 12.1 Content Completeness

- [ ] Message identifies **what failed** (operation, assertion, validation)
- [ ] Message names the **target/subject** (file, field, component)
- [ ] Message includes **expected state** when relevant
- [ ] Message includes **actual state** when it differs
- [ ] Message includes **condition** when behavior is conditional

### 12.2 Context Appropriateness

- [ ] **File context** included when relevant (I/O operations, config)
- [ ] **Component context** included when multiple components could fail
- [ ] **Field context** included for validation errors
- [ ] **Function context** included when operation is non-obvious

### 12.3 Actionability Appropriateness

- [ ] Action suggestions are **specific and accurate**
- [ ] Action suggestions are **safe to follow**
- [ ] Diagnostic hints **guide investigation effectively**
- [ ] No **speculative or vague suggestions**
- [ ] **Audience-appropriate depth** (user vs developer)

### 12.4 Format Compliance

- [ ] Follows **"should"/"Failed to"/"must"** patterns
- [ ] **Preserves original case** for identifiers
- [ ] Uses **`{:?}` for debug**, **`{}` for display**
- [ ] **No trailing period**
- [ ] **Placeholders at end** of message
- [ ] **No unnecessary quotes** around simple values
- [ ] **First word capitalized**

### 12.5 Self-Documentation

- [ ] Message is **understandable without reading code**
- [ ] Message uses **consistent terminology** with codebase
- [ ] Test assertions are **self-documenting**
- [ ] Error path validations are **clear about expectations**

---

## 13. Migration Path

### 13.1 Phase 1: Add Minimum Context

Add basic context to bare messages:

```rust
// Before
.unwrap()

// After
.expect("Failed to read config")
```

### 13.2 Phase 2: Add Cause and Context

Add more diagnostic information:

```rust
// After
.expect("Failed to read config from ~/.hoop/config.yml")
```

### 13.3 Phase 3: Add Actionable Suggestions

Add safe, obvious fixes:

```rust
// After
.expect("Failed to read config from ~/.hoop/config.yml. Run 'hoop init' to create config")
```

### 13.4 Priority Order

1. **Critical errors first** — unwrap() in production code paths
2. **Test assertions** — add context to all test failures
3. **User-facing errors** — CLI and API messages
4. **Internal diagnostics** — developer-facing debugging aids

---

## Appendix: Pattern Summary

### Sentence Patterns

| Pattern | Use When | Example |
|---------|----------|---------|
| `<subject> should <state>` | Expected behavior | `flag should be true when --flag present` |
| `Failed to <action> <target>` | Operation failures | `Failed to read config from path` |
| `<subject> must <condition>` | Invariants | `file must exist before start` |
| `Expected <expected>, got <actual>` | Value mismatches | `Expected string, got integer` |
| `<action> should <result>` | Validation tests | `missing field should fail` |

### Context Rules

| Context | When to Include | Example |
|---------|----------------|---------|
| File path | I/O, config, user files | `from ~/.hoop/config.yml` |
| Function name | Public API, ambiguous | `ConfigParser::parse() failed` |
| Line/column | Parsing errors | `at line 15, column 3` |
| Field name | Validation errors | `schema_version must be string` |
| Component | Multiple components | `Daemon should start` |

### Actionability Levels

| Level | When | Example |
|-------|------|---------|
| Informational-only | Internal, diagnostic | `Failed to deserialize bead line` |
| Diagnostic hint | Complex errors | `Hint: Check line format matches...` |
| Direct action | User-facing, safe fix | `Run 'hoop init' to create config` |
| Documentation | Complex recovery | `See docs/operations.md for steps` |

---

**Document Status:** Complete  
**Related Documents:**
- [Error Message Catalog](../error_messages_catalog.md) — Current inventory
- [AGENTS.md](../AGENTS.md) — Repository guide for LLMs

**Next Steps:** Apply these standards in error message improvement work across HOOP codebase. Use the validation checklist (Section 12) before committing any new error messages.
