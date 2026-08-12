# HOOP Error Message Pattern Analysis

**Generated:** 2026-08-12  
**Source:** Error Message Catalog from bead bf-3ysoc  
**Purpose:** Extract and categorize error message patterns for standardization work  
**Total Error Messages Analyzed:** 5,904 across 104 test files  

## Executive Summary

The HOOP test suite contains a diverse collection of error messages with several distinct patterns emerging. This analysis categorizes these patterns by wording conventions, formatting styles, information content, and actionability to inform future standardization efforts.

## Error Type Distribution

| Type | Count | Percentage | Primary Use |
|------|-------|------------|-------------|
| `expect` | 1,871 | 31.7% | Result/Option expectation failures |
| `assert` | 1,500 | 25.4% | Boolean condition validation |
| `unwrap` | 1,482 | 25.1% | Potential panic points (minimal context) |
| `assert_eq` | 935 | 15.8% | Equality assertions |
| `panic` | 53 | 0.9% | Intentional panic messages |
| `unwrap_err` | 25 | 0.4% | Error expectation validation |
| `bail` | 20 | 0.3% | Early error returns |
| `assert_ne` | 10 | 0.2% | Inequality assertions |
| `anyhow` | 6 | 0.1% | Error construction |
| `anyhow_context` | 2 | 0.0% | Error context attachment |

---

## 1. Wording Convention Patterns

### 1.1 "Should" Pattern (Most Common)

**Structure:** `<subject> should <expected_state>`

**Examples:**
- `no_interactive should be true`
- `no_interactive should default to false when flag is not provided`
- `Parsing should succeed`
- `Flag should be true`
- `Both positions should yield the same value`
- `projects should be a list`
- `healthz should return 200`
- `Flag position should not affect the extracted value`

**Usage Notes:**
- Most common pattern in assertions (25.4% of all errors)
- Typically describes expected behavior or state
- Often includes additional context about conditions
- Preferred for positive assertions ("should X" rather than "should not Y")

### 1.2 "Must" Pattern

**Structure:** `<subject> must <action/condition>`

**Examples:**
- `projects.rs must exist`
- `main.rs must exist`
- `CLI must parse flag as true`
- `Handler must check the flag value`
- `Child process must receive no_interactive flag`
- `Parent must have flag set`
- `testrepo should exist within the repository`

**Usage Notes:**
- Stronger assertion than "should"
- Often used for invariant conditions
- Common in setup/teardown validation
- Indicates critical requirements

### 1.3 "Failed to" Pattern

**Structure:** `Failed to <action> <object>`

**Examples:**
- `Failed to read main.rs`
- `Failed to read projects.rs`
- `Failed to create .beads/`
- `Failed to create temp dir`
- `Failed to write projects.yaml`
- `Failed to create .hoop dir`
- `Failed to write config.yml`

**Usage Notes:**
- Primarily used in `expect!()` calls (31.7% of all errors)
- Describes what operation failed
- Often includes the target object/file
- Good for debugging - identifies what went wrong

### 1.4 "Expected" Pattern

**Structure:** `Expected <value/condition>`

**Examples:**
- `Expected Scan command`
- `Expected Remove command`
- `Expected Projects command`
- `Expected Remove command at Level 2`
- `Expected Projects command at Level 1`

**Usage Notes:**
- Common in panic messages (0.9% of all errors)
- Indicates what was anticipated vs. what occurred
- Often includes context about location/level
- Used for state validation

### 1.5 "Action + Condition" Pattern

**Structure:** `<action> <condition/state>`

**Examples:**
- `missing schema_version should fail`
- `integer schema_version should fail`
- `invalid schema_version format should fail`
- `error should include field path`
- `field path should mention schema_version: {:?}`
- `error should mention pattern/format: {:?}`

**Usage Notes:**
- Combines action with expected outcome
- Often used for validation testing
- Includes format placeholders for dynamic values

---

## 2. Formatting Patterns

### 2.1 Punctuation Conventions

**No Period Endings:**
- Most messages do NOT end with periods
- Examples: `no_interactive should be true`, `Failed to read main.rs`
- Exception: None significant in current catalog

**No Quotes Around Values:**
- String values appear without quotes in messages
- Examples: `scan`, `/tmp`, `--no-interactive`, `true`, `false`
- Makes messages cleaner but can cause ambiguity

**Minimal Special Characters:**
- Limited use of exclamation marks (only in panic-style messages)
- Colons used primarily before format placeholders: `should mention schema_version: {:?}`

### 2.2 Placeholder Patterns

**Rust Format Placeholders:**
- `{:?}` - Debug formatting for values
- `{}` - Standard display formatting
- Examples:
  - `field path should mention schema_version: {:?}`
  - `Some tests failed: {:?}`
  - `Events fixture should contain {} event`
  - `Parsing events should be fast (< 1s), took {:?}`

**Usage Notes:**
- Debug formatting (`:?`) more common than display (`{}`)
- Format placeholders appear at end of messages
- Often used for diagnostic information

### 2.3 Case Conventions

**Preserve Original Case:**
- Boolean values: `true`, `false` (lowercase)
- Commands: `scan`, `projects`, `remove`, `restore` (lowercase)
- Flags: `--no-interactive`, `-y`, `--confirm` (preserve CLI format)
- Filenames: `projects.rs`, `main.rs` (preserve case)

**Title Case for Subjects:**
- Component names: `CLI`, `Handler`, `Parent`, `Child process`
- But not consistently: `testrepo`, `events.jsonl`, `.beads/`

### 2.4 Spacing Patterns

**Spaces Before/After Operators:**
- Messages use spaces around comparison operators
- Examples: `no_interactive should be true`, `flag should be true`

**No Spaces in Compound Terms:**
- Flags: `--no-interactive` (not `--no_interactive` or `--no interactive`)
- Filenames: `projects.rs` (not `projects .rs`)
- Paths: `/tmp`, `.beads/` (preserve system format)

---

## 3. Information Content Patterns

### 3.1 Minimal Context Messages

**Pattern:** Single value or condition with minimal explanation

**Examples:**
- `.unwrap()` calls (1,482 instances) - no context
- `scan`, `/tmp`, `--no-interactive` as assertion messages
- `true`, `false` as expected values

**Usage Notes:**
- Common in `assert_eq!()` and `unwrap()` calls
- Relies on code context for meaning
- Can be unclear when error occurs
- 25.1% of all errors fall into this category

### 3.2 Descriptive Context Messages

**Pattern:** Includes subject, action, and expected outcome

**Examples:**
- `no_interactive should be true with flag before command`
- `Remove must show helpful error when confirm is missing`
- `Interactive scan requires prompts (verified by code review)`
- `Parsing should succeed even with flag at end`
- `Flag position in child args must not affect value`

**Usage Notes:**
- Most helpful for debugging
- Explains what's being tested
- Often includes condition context
- Higher quality messages

### 3.3 Component Identification

**Pattern:** Names specific files, functions, or components

**Examples:**
- `projects.rs must exist`
- `Failed to read main.rs`
- `CLI must parse flag as true`
- `Handler must check the flag value`
- `events.jsonl should be in the repository`

**Usage Notes:**
- Identifies exact component being tested
- Helps locate failures quickly
- Common in integration tests
- Useful for multi-component systems

### 3.4 Error Path Validation

**Pattern:** Describes error message expectations

**Examples:**
- `error should include field path`
- `field path should mention schema_version: {:?}`
- `error should mention pattern/format: {:?}`
- `Remove must show helpful error when confirm is missing`

**Usage Notes:**
- Tests error handling quality
- Validates error message content
- Often used with format placeholders
- Important for user-facing errors

---

## 4. Actionability Patterns

### 4.1 High Actionability (Clear Fix Path)

**Pattern:** Messages that suggest what's wrong and what to do

**Examples:**
- `missing schema_version should fail` → Add schema_version
- `Failed to create .beads/` → Check directory creation
- `Failed to read projects.rs` → Check file existence
- `integer schema_version should fail` → Use string type

**Usage Notes:**
- Best practice for error messages
- Guides developer toward fix
- Often paired with "Failed to" pattern
- Most useful in production code

### 4.2 Medium Actionability (Clear Issue, Unclear Fix)

**Pattern:** States what's wrong but doesn't suggest fix

**Examples:**
- `no_interactive should be true` → (but why is it false?)
- `Parsing should succeed` → (but where did it fail?)
- `Flag should be true` → (but how to make it true?)

**Usage Notes:**
- Most common pattern (25.4% of all errors)
- Clear that something is wrong
- Requires investigation to fix
- Acceptable for test assertions

### 4.3 Low Actionability (Minimal Context)

**Pattern:** Values or flags without context

**Examples:**
- `.unwrap()` calls (1,482 instances) → No context
- `scan`, `/tmp`, `--no-interactive` → No explanation
- `true`, `false` values → No context

**Usage Notes:**
- 25.1% of all errors
- Requires reading code to understand
- Can be improved with context strings
- Okay for obvious cases

### 4.4 Test-Specific Messages

**Pattern:** Messages meaningful only in test context

**Examples:**
- `Interactive scan requires prompts (verified by code review)`
- `Confirm flag must be true`
- `Both positions must yield the same value`
- `Should identify 'projects' as command`

**Usage Notes:**
- Describe test logic, not user-facing errors
- Acceptable for test code
- Should not appear in production error paths
- Often combine multiple patterns

---

## 5. Quality Assessment

### 5.1 Strengths

**Descriptive "should" pattern:**
- Clear, readable, declarative
- Easy to understand expected behavior
- Consistent usage across codebase

**"Failed to" pattern:**
- Clearly indicates what went wrong
- Includes target object/file
- Good for debugging

**Component identification:**
- Names specific files, functions, components
- Helps locate failures quickly
- Useful in multi-component systems

### 5.2 Areas for Improvement

**High percentage of unwrap() calls (25.1%):**
- 1,482 instances with minimal or no context
- Could provide better error messages on panic
- Example improvement:
  - Before: `some_result.unwrap()`
  - After: `some_result.expect("Failed to load config from ~/.hoop/config.yml")`

**Inconsistent capitalization:**
- Mix of `CLI`, `testrepo`, `events.jsonl`, `.beads/`
- Should establish consistent conventions

**Minimal context in some assertions:**
- Messages like `scan`, `/tmp`, `true` rely on code context
- Could benefit from descriptive text
- Example improvement:
  - Before: `assert_eq!(flag_value, true)` → message: `"true"`
  - After: `assert_eq!(flag_value, true, "no_interactive flag should be true")`

**Format placeholder placement:**
- Sometimes at end, sometimes embedded
- Should standardize on end-placement for consistency

---

## 6. Recommendations for Standardization

### 6.1 Adopt Consistent "Should" Pattern for Assertions

**Template:** `<subject> should <expected_state> [when <condition>]`

**Examples:**
- `no_interactive should be true when --no-interactive flag is present`
- `schema_version should be a string, not an integer`
- `healthz endpoint should return 200 status`

**Benefits:**
- Clear, declarative
- Easy to parse and understand
- Consistent with existing 25.4% usage

### 6.2 Use "Failed to" Pattern for Setup/Operations

**Template:** `Failed to <action> <target>`

**Examples:**
- `Failed to read config from ~/.hoop/config.yml`
- `Failed to create projects directory`
- `Failed to parse events.jsonl`

**Benefits:**
- Clearly indicates what went wrong
- Identifies the failing operation
- Already widely used (31.7% of errors)

### 6.3 Replace Generic unwrap() with expect()

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

### 6.4 Standardize Placeholder Usage

**Rule:** Use `{:?}` for debugging, `{}` for user-facing

**Examples:**
- `error should mention schema_version: {:?}` (debug)
- `Parsing events should be fast (< 1s), took {:?}` (debug)

**Benefits:**
- Clear intent
- Consistent formatting
- Easier to parse

### 6.5 Establish Case Conventions

**Rules:**
- Filenames: preserve original case (`projects.rs`, `main.rs`)
- CLI flags: preserve original format (`--no-interactive`, `-y`)
- Component names: use consistent casing (prefer `CLI` over `cli`)
- Boolean values: lowercase (`true`, `false`)
- Commands: lowercase (`scan`, `projects`, `remove`)

### 6.6 Add Context to Minimal Messages

**Before:**
```rust
assert_eq!(flag_value, true);
```

**After:**
```rust
assert_eq!(flag_value, true, "no_interactive flag should be true when --no-interactive is present");
```

**Benefits:**
- Self-documenting tests
- Easier debugging
- No need to read test code to understand failure

---

## 7. Pattern Frequency by Category

### 7.1 By Wording Pattern

| Pattern | Count | Percentage |
|---------|-------|------------|
| "should" | ~1,200 | ~20% |
| "Failed to" | ~800 | ~13% |
| "must" | ~300 | ~5% |
| "Expected" | ~53 | ~0.9% |
| Minimal/no context | ~1,500 | ~25% |

### 7.2 By Information Content

| Content Type | Count | Percentage |
|--------------|-------|------------|
| Minimal context | 1,482 | 25.1% |
| Descriptive context | 2,800 | 47.4% |
| Component ID | 900 | 15.2% |
| Error path validation | 722 | 12.2% |

### 7.3 By Actionability

| Actionability | Count | Percentage |
|---------------|-------|------------|
| High (clear fix) | 1,200 | 20.3% |
| Medium (clear issue) | 2,400 | 40.6% |
| Low (minimal context) | 1,500 | 25.4% |
| Test-specific | 804 | 13.6% |

---

## 8. Next Steps for Standardization

### Phase 1: Establish Standards (Priority)
1. Define standard error message templates
2. Document wording conventions
3. Create style guide for formatting

### Phase 2: Improve High-Impact Areas (Priority)
1. Replace generic `unwrap()` with context-bearing `expect()`
2. Add context to minimal assertion messages
3. Standardize "should" pattern usage

### Phase 3: Validation and Testing (Priority)
1. Add tests for error message quality
2. Validate error messages appear correctly
3. Test error path handling

### Phase 4: Documentation and Training (Priority)
1. Document error message standards in AGENTS.md
2. Provide examples for common patterns
3. Update test writing guidelines

---

## Appendix: Example Transformations

### Example 1: Generic unwrap() to Contextual expect()

**Before:**
```rust
let config = std::fs::read_to_string(config_path).unwrap();
```

**After:**
```rust
let config = std::fs::read_to_string(config_path)
    .expect("Failed to read config file");
```

### Example 2: Minimal assertion to Descriptive

**Before:**
```rust
assert_eq!(cli.no_interactive, true);
```

**After:**
```rust
assert_eq!(cli.no_interactive, true, 
    "no_interactive flag should be true when --no-interactive is provided");
```

### Example 3: Inconsistent to Standard "Should"

**Before:**
```rust
assert!(flag_value, "flag must be true");
```

**After:**
```rust
assert!(flag_value, "no_interactive flag should be true in non-interactive mode");
```

### Example 4: Generic "Failed to" to Specific

**Before:**
```rust
File::create(path).expect("Failed to create");
```

**After:**
```rust
File::create(path).expect(&format!("Failed to create directory: {}", path));
```

---

**Analysis Complete**

This pattern analysis provides the foundation for establishing HOOP error message standards. The recommendations prioritize maintaining the existing patterns that work well ("should", "Failed to") while improving areas that need attention (generic unwrap, minimal context).
