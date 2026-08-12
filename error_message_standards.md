# HOOP Error Message Standards

**Generated:** 2026-08-12  
**Source:** Pattern analysis from error message catalog (bf-3du8o)  
**Purpose:** Define wording and formatting standards for error messages across HOOP  
**Status:** Standards for bead bf-4qy5x

## Overview

This document defines the standards for error message wording conventions and formatting patterns in the HOOP project. These standards derive from existing patterns found across 5,904 error messages in the test suite, ensuring consistency while minimizing disruptive changes.

## Core Principles

1. **Clarity over brevity** - Prefer descriptive messages that explain what happened and why
2. **Actionability** - Error messages should guide the developer/user toward a fix
3. **Consistency** - Follow established patterns across the codebase
4. **Context preservation** - Maintain original casing for CLI flags, filenames, and identifiers

---

## 1. Sentence Structure Standards

### 1.1 Preferred Patterns (in priority order)

#### Pattern A: "Should" Pattern (Primary)

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
```

**Rationale:** The "should" pattern is the most common in the codebase (25.4% of errors), is declarative and easy to understand, and clearly expresses expected behavior.

#### Pattern B: "Failed to" Pattern (Operations)

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
.expect("failed reading") // what failed?
.expect("Failed") // failed to do what?
```

**Rationale:** Already widely used (31.7% of errors), clearly identifies what went wrong and the target, making debugging faster.

#### Pattern C: "Must" Pattern (Invariants)

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
"projects.rs should exist" // "should" is for preferences, "must" for requirements
```

**Rationale:** "Must" expresses stronger assertions than "should" and should be reserved for invariants and critical requirements.

### 1.2 Conditional Phrasing

**Standard:** Use `when <condition>` for contextual information

**Examples:**
```rust
// ✅ Correct
"no_interactive should be true when --no-interactive flag is present"
"parsing should succeed even when flag is at end of command"

// ❌ Avoid
"no_interactive should be true with flag" // what flag?
"flag should be true in this case" // what case?
```

**Rationale:** Explicit `when` clauses make conditions clear and unambiguous.

### 1.3 Action + Outcome Pattern

**Structure:** `<action> should <result>`

**Usage:**
- Validation testing
- Error path verification
- Command behavior testing

**Examples:**
```rust
// ✅ Correct
"missing schema_version should fail validation"
"invalid schema_version format should return error"
"error should include field path"

// ❌ Avoid
"missing schema_version fails" // fails what?
"error has field path" // should include specific field
```

**Rationale:** Combines action with expected outcome, making test intent clear.

---

## 2. Punctuation Standards

### 2.1 Periods

**Standard:** Do NOT end error messages with periods

**Examples:**
```rust
// ✅ Correct
.expect("Failed to read config")
.assert_eq!(value, true, "flag should be true")

// ❌ Avoid
.expect("Failed to read config.") // trailing period
.assert_eq!(value, true, "flag should be true.") // trailing period
```

**Rationale:** Error messages are fragments, not sentences. Periods add visual clutter without meaning. This is already the dominant pattern in the codebase.

### 2.2 Quotes Around Values

**Standard:** Do NOT use quotes around simple values unless necessary for clarity

**When to omit quotes:**
- Boolean values: `true`, `false`
- Commands: `scan`, `projects`, `remove`
- CLI flags: `--no-interactive`, `-y`
- Simple strings with no spaces: `schema_version`, `.beads/`

**When to use quotes:**
- Values containing spaces: `"error message"`
- Ambiguous strings: `"true"` (as a string, not boolean)
- User-facing text: `"Please try again"`

**Examples:**
```rust
// ✅ Correct - no quotes needed
"no_interactive should be true"
"command should be scan"
"path should be .beads/"

// ✅ Correct - quotes for clarity
"error message should contain 'invalid'"
"expected 'true', got 'false' string"

// ❌ Avoid - unnecessary quotes
"no_interactive should be 'true'"
"command should be 'scan'"
```

**Rationale:** Unnecessary quotes clutter messages. Use only when needed to disambiguate strings from keywords or to highlight user-facing text.

### 2.3 Commas

**Standard:** Use commas before clauses and in lists

**Examples:**
```rust
// ✅ Correct
"Failed to read config, but daemon should continue"
"file, path, and line should all be present"

// ❌ Avoid
"Failed to read config but daemon should continue" // missing comma
"file path and line should all be present" // missing serial comma
```

**Rationale:** Commas improve readability and make complex messages parseable.

### 2.4 Colons

**Standard:** Use colons before format placeholders or explanations

**Examples:**
```rust
// ✅ Correct
"field path should mention: schema_version"
"error should mention pattern: {:?}"
"Failed to read: {}"

// ❌ Avoid
"field path should mention schema_version" // colon would be clearer
"error should mention pattern {:?}" // colon missing before placeholder
```

**Rationale:** Colons signal that what follows is a value, explanation, or formatted output.

---

## 3. Capitalization Conventions

### 3.1 Component Names

**Standard:** Use consistent casing for component types

**Rules:**
- Acronyms/initialisms: Uppercase (CLI, API, HTTP, JSON, YAML)
- Components: Title case (Daemon, Handler, Manager)
- Generic terms: lowercase (daemon, handler, manager) - avoid unless specific

**Examples:**
```rust
// ✅ Correct
"CLI should parse flag correctly"
"Daemon should start without errors"
"API should return 200 status"

// ⚠️ Context-dependent
"handler should check" // which handler? prefer "ConfigHandler should check"
"daemon should restart" // which daemon? prefer "HOOP daemon should restart"
```

**Rationale:** Consistent capitalization makes components identifiable. Acronyms are conventionally uppercase.

### 3.2 Preserve Original Case

**Standard:** Preserve original casing for system identifiers

**Rules:**
- Filenames: preserve case (`projects.rs`, `main.rs`, `File`)
- CLI flags: preserve format (`--no-interactive`, `-y`, `--confirm`)
- Commands: lowercase (`scan`, `projects`, `remove`)
- Directories: preserve format (`.beads/`, `.hoop/`, `/tmp/`)
- Boolean values: lowercase (`true`, `false`)

**Examples:**
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

**Rationale:** Preserving original case makes identifiers searchable and prevents confusion.

### 3.3 First Word Capitalization

**Standard:** Capitalize the first word of the message

**Examples:**
```rust
// ✅ Correct
"Failed to read config"
"CLI should parse flag"
"schema_version should be a string"

// ❌ Avoid
"failed to read config" // lowercase first word
"cLI should parse flag" // incorrect casing
```

**Rationale:** Sentence case makes messages easier to read and scan.

---

## 4. Actual vs Expected Value Presentation

### 4.1 Standard Format

**Structure:** `Expected <expected>, got <actual>` or `expected: <expected>, actual: <actual>`

**Usage:**
- Type mismatches
- Value comparisons
- Format validation

**Examples:**
```rust
// ✅ Correct - with labels
"Expected string, got integer"
"expected: true, actual: false"
"Expected 200 status, got 500"

// ✅ Correct - format placeholders
"Expected {}, got {:?}", expected, actual
"expected: {:?}, actual: {:?}", expected, actual

// ❌ Avoid - unclear ordering
"true vs false" // which is expected?
"integer but got string" // awkward phrasing
```

**Rationale:** Explicit "expected" and "actual" labels remove ambiguity about which value is which.

### 4.2 Format Placeholder Usage

**Standard:** Use `{:?}` for debugging, `{}` for user-facing

**Debug formatting (`{:?}`):**
- Internal diagnostics
- Developer-facing errors
- Complex types (structs, enums)
- Test assertion failures

**Display formatting (`{}`):**
- User-facing messages
- Simple primitive values
- Output intended for end users

**Examples:**
```rust
// ✅ Correct - debug formatting
"Failed to parse config: {:?}", error
"field path should mention: {:?}", actual_path

// ✅ Correct - display formatting
"Failed to open file: {}", filename
"Expected command: {}, got: {}", expected_cmd, actual_cmd

// ❌ Avoid - mixing user intent with debug output
"Failed to read: {:?}" // user doesn't need debug format
"error message: {}" // developer needs more context
```

**Rationale:** Debug formatting provides maximum information for developers. Display formatting is cleaner for user-facing output.

### 4.3 Placeholder Placement

**Standard:** Place format placeholders at the end of messages

**Examples:**
```rust
// ✅ Correct - placeholder at end
"Failed to read config: {}", filename
"field path should mention: {:?}", path

// ✅ Correct - multiple placeholders at end
"Expected {}, got {}", expected, actual
"Failed to read {} from {}", file, directory

// ❌ Avoid - embedded placeholders
"Failed {} to read config" // awkward
"field path {} should mention: {:?}", path, value // confusing
```

**Rationale:** End placement makes templates easier to read and parse.

---

## 5. Special Cases

### 5.1 Test-Specific Messages

**Standard:** Test messages can reference test logic but should be self-documenting

**Examples:**
```rust
// ✅ Correct - test-specific but clear
"Interactive scan requires prompts (verified by code review)"
"Both positions must yield the same value"
"Should identify 'projects' as command"

// ❌ Avoid - cryptic test references
"test case 3 failed" // what test?
"verify behavior" // what behavior?
```

**Rationale:** Test messages should be understandable without reading test code.

### 5.2 Error Path Validation Messages

**Standard:** Describe what the error message should contain

**Structure:** `error should include <content>` or `error should mention <content>`

**Examples:**
```rust
// ✅ Correct
"error should include field path"
"error should mention schema_version: {:?}"
"error message should contain 'invalid'"

// ❌ Avoid
"error has field path" // should include what?
"error mentions schema_version" // should be clearer
```

**Rationale:** Testing error messages requires clear expectations about content.

### 5.3 unwrap() → expect() Migration

**Standard:** Replace all `unwrap()` calls with context-bearing `expect()`

**Before:**
```rust
let config = File::open("config.yml").unwrap();
```

**After:**
```rust
let config = File::open("config.yml")
    .expect("Failed to open config.yml for reading");
```

**Benefits:**
- Provides context on panic
- Easier debugging
- Maintains same panic behavior
- Self-documenting code

---

## 6. Minimal Context Improvements

### 6.1 Assertion Messages

**Standard:** Always provide descriptive messages for assertions

**Before:**
```rust
assert_eq!(flag_value, true);
assert!(result.is_ok());
```

**After:**
```rust
assert_eq!(flag_value, true, 
    "no_interactive flag should be true when --no-interactive is present");
assert!(result.is_ok(), 
    "operation should succeed: {:?}", result);
```

**Benefits:**
- Self-documenting tests
- Easier debugging
- No need to read test code to understand failure

### 6.2 Boolean Assertions

**Standard:** Use descriptive "should be" patterns for booleans

**Before:**
```rust
assert!(flag_value, "flag must be true");
assert_eq!(no_interactive, true);
```

**After:**
```rust
assert!(flag_value, 
    "no_interactive flag should be true in non-interactive mode");
assert_eq!(no_interactive, true, 
    "no_interactive should be true when --no-interactive is provided");
```

**Benefits:**
- Clearer than bare boolean values
- Explains context and condition

---

## 7. Complete Examples

### Example 1: File Operation Error

**❌ Poor:**
```rust
let config = std::fs::read_to_string(path).unwrap();
```

**✅ Good:**
```rust
let config = std::fs::read_to_string(path)
    .expect(&format!("Failed to read config from: {}", path));
```

**Improvements:**
- Contextual error message
- Includes failing path
- Uses "Failed to" pattern

### Example 2: Validation Test

**❌ Poor:**
```rust
assert_eq!(result.is_err(), true);
```

**✅ Good:**
```rust
assert!(result.is_err(), 
    "missing schema_version should fail validation");
```

**Improvements:**
- Descriptive message
- Action + outcome pattern
- Explains validation intent

### Example 3: CLI Parsing

**❌ Poor:**
```rust
assert_eq!(cli.no_interactive, true);
```

**✅ Good:**
```rust
assert_eq!(cli.no_interactive, true, 
    "no_interactive should be true when --no-interactive flag is present");
```

**Improvements:**
- Condition context
- Component identification
- Standard "should" pattern

### Example 4: Type Mismatch

**❌ Poor:**
```rust
panic!("expected string, got integer");
```

**✅ Good:**
```rust
panic!("Expected schema_version to be string, got integer: {:?}", actual);
```

**Improvements:**
- Explicit "Expected/got" pattern
- Includes actual value for debugging
- Identifies the field

### Example 5: Invariant Violation

**❌ Poor:**
```rust
assert!(exists, "file must exist");
```

**✅ Good:**
```rust
assert!(exists, 
    "projects.rs must exist in the repository before daemon starts");
```

**Improvements:**
- "Must" for invariant
- Contextual condition
- Component identification

---

## 8. Anti-Patterns to Avoid

### 8.1 Cryptic Messages

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

### 8.2 Over-Vague References

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

### 8.3 Missing Context

**❌ Avoid:**
```rust
.unwrap()
.expect("failed")
assert_eq!(value, expected)
```

**✅ Instead:**
```rust
.expect("Failed to read config from ~/.hoop/config.yml")
.expect(&format!("Failed to create directory: {}", path))
assert_eq!(value, expected, "field should match expected value: {:?}", expected)
```

### 8.4 Inconsistent Terminology

**❌ Avoid:**
```rust
"cli should parse" // sometimes CLI, sometimes cli
"testrepo must exist" // sometimes testRepo, sometimes testrepo
"--no_interactive" // sometimes --no-interactive
```

**✅ Instead:**
```rust
"CLI should parse" // always uppercase for acronym
"testrepo must exist" // consistent casing
"--no-interactive" // preserve original format
```

---

## 9. Migration Checklist

When updating error messages to meet these standards:

- [ ] Replace `unwrap()` with `expect()` containing descriptive context
- [ ] Add descriptive messages to all assertions
- [ ] Use "should" pattern for expected behavior
- [ ] Use "Failed to" pattern for operations
- [ ] Use "must" pattern for invariants
- [ ] Preserve original case for identifiers
- [ ] Place format placeholders at end of messages
- [ ] Use `{:?}` for debug, `{}` for display
- [ ] Remove trailing periods
- [ ] Remove unnecessary quotes
- [ ] Add condition context with "when" clauses
- [ ] Use "Expected/got" for value comparisons

---

## 10. Quality Checklist

Before committing new error messages, verify:

- [ ] Message is self-documenting (clear without reading code)
- [ ] Message identifies what failed (operation, assertion, validation)
- [ ] Message includes relevant context (file, field, condition)
- [ ] Message follows one of the standard patterns
- [ ] Placeholder values are at the end
- [ ] Original case is preserved for identifiers
- [ ] No trailing period
- [ ] Quotes used only when necessary
- [ ] Debug format (`:?`) used for complex values
- [ ] Display format (`{}`) used for user-facing output

---

## Appendix: Quick Reference

### Pattern Templates

| Purpose | Template | Example |
|---------|----------|---------|
| Expected behavior | `<subject> should <state> [when <condition>]` | `no_interactive should be true when --no-interactive is present` |
| Operation failure | `Failed to <action> <target>` | `Failed to read config from ~/.hoop/config.yml` |
| Invariant | `<subject> must <condition>` | `projects.rs must exist in the repository` |
| Value comparison | `Expected <expected>, got <actual>` | `Expected string, got integer` |
| Validation test | `<action> should <result>` | `missing schema_version should fail validation` |
| Error content | `error should include <content>` | `error should include field path` |

### Format Placeholder Rules

| Format | Usage | Example |
|-------|-------|---------|
| `{:?}` | Debug, developer-facing | `Failed to parse: {:?}", error |
| `{}` | Display, user-facing | `Failed to open file: {}", filename` |

### Capitalization Rules

| Type | Rule | Example |
|------|------|---------|
| Acronyms | Uppercase | `CLI`, `API`, `HTTP` |
| Components | Title case | `Daemon`, `Handler`, `Manager` |
| Filenames | Preserve case | `projects.rs`, `main.rs` |
| CLI flags | Preserve format | `--no-interactive`, `-y` |
| Commands | Lowercase | `scan`, `projects`, `remove` |
| Booleans | Lowercase | `true`, `false` |

---

**Document Status:** Complete - Standards defined for bead bf-4qy5x  
**Next Steps:** Apply these standards in error message improvement work
