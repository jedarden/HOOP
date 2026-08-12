# HOOP Error Message Consistency Standards

**Generated:** 2026-08-12  
**Purpose:** Comprehensive standards for consistent, actionable, and clear error messages across the HOOP test suite and codebase  
**Derived from:** Analysis of 5,904 error message patterns across 104 test files (bead bf-3ysoc, bf-3du8o, bf-4qy5x, bf-4ory0)  

---

## Table of Contents

1. [Principles](#principles)
2. [Minimum Informational Content](#minimum-informational-content)
3. [Wording Conventions](#wording-conventions)
4. [Formatting Patterns](#formatting-patterns)
5. [Context Inclusion Guidelines](#context-inclusion-guidelines)
6. [Actionability Guidelines](#actionability-guidelines)
7. [Error Type Standards](#error-type-standards)
8. [Complete Examples](#complete-examples)
9. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
10. [Quality Checklist](#quality-checklist)

---

## Principles

### Core Principles

1. **Clarity First** - Error messages must be immediately understandable to developers encountering test failures, even without deep context
2. **Complete Context** - Include all information needed to understand and diagnose the error
3. **Actionability** - When appropriate, provide clear next steps or solution hints
4. **Consistency** - Follow the same patterns across similar assertion types for predictable, scannable output
5. **Audience Awareness** - Distinguish between developer-facing and user-facing messages

---

## Minimum Informational Content

### Universal Requirements (Every Error Message)

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

### Conditional Information

#### D. Condition Context (When Behavior is Conditional)

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

#### E. Actual Values (When Different from Expected)

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

## Wording Conventions

### Standard Phrasing Patterns

| Pattern | Standard Form | Example |
|---------|--------------|---------|
| Expected behavior | `<subject> should <state> [when <condition>]` | `"no_interactive should be true when --no-interactive is present"` |
| Negative expectation | `<subject> should not <verb>` | `"Should not accept invalid input"` |
| Operation failure | `Failed to <action> <target> [because <reason>]` | `"Failed to read config: file not found"` |
| Invariant/requirement | `<subject> must <condition>` | `"projects.rs must exist in repository"` |
| Value comparison | `Expected <expected>, got <actual>` | `"Expected string, got integer"` |
| HTTP status | `<endpoint> should return <status>` | `"healthz should return 200"` |
| Field validation | `<invalid> <field> should fail` | `"missing schema_version should fail"` |
| Type checking | `<noun> should be <type>` | `"projects should be a list"` |

### Sentence Structure

**Preferred Order:** [Subject] + [Expected State] + [Context]

```
✅ Good: "Daemon should be healthy after boot"
✅ Good: "Fetched bead ID should match"
✅ Good: "All WebSocket connections should receive init"

❌ Avoid: "bead id should not be empty" (inconsistent capitalization)
❌ Avoid: "projects should be a list" (missing article "a")
```

### Capitalization and Punctuation

- **Sentence case:** First word capitalized, others lowercase (except proper nouns)
- **No trailing period:** Messages should not end with `.`
- **Use articles:** Include `a`, `an`, `the` for readability
- **Preserve original case:** For identifiers, flags, commands (e.g., `--no-interactive`, not `--no_interactive`)

```
✅ Good: "projects should be a list"
✅ Good: "Daemon should be healthy"
✅ Good: "metrics should contain at least one valid metric line"

❌ Avoid: "Projects should be a List" (random capitalization)
❌ Avoid: "should have 2 open beads." (trailing period)
❌ Avoid: "metrics should contain valid metric line" (missing articles)
```

### Comparison Messaging Order

When comparing values, use **expected vs. actual** order consistently:

```
✅ Good: "expected 200 OK, got 404 Not Found"
✅ Good: "expected flag to be true, found false"
✅ Good: "expected 'scan', got 'status'"

❌ Avoid: "got 404 instead of 200" (inconsistent order)
❌ Avoid: "found false but expected true" (reversed order)
```

### Verbs to Use

**For assertions/expectations:**
- `should` - preferred for assertions (e.g., `"should be true"`)
- `must` - for invariants and requirements (e.g., `"must exist"`)
- `Failed to` - for operation failures (e.g., `"Failed to read"`)

**Avoid:**
- `would` - too speculative
- `could` - unclear intent
- `might` - not diagnostic

---

## Formatting Patterns

### Quoting and Literals

**Use quotes for:**
- String values: `expected "scan", got "status"`
- Field names: `"no_interactive" flag should be true`
- Command names: `'scan'`, `'remove'`, `'--no-interactive'`

**Don't use quotes for:**
- Types: `should be a list`, `should be an object`
- Booleans: `flag should be true` (not `flag should be "true"`)
- Numbers: `should have 2 open beads`
- Paths: `~/.hoop/config.yml` (not `"~/.hoop/config.yml"`)

```rust
✅ Good: assert_eq!(parsed.subcommand, Some("scan".to_string()), 
                   "expected 'scan', got something else");
✅ Good: assert!(projects.is_array(), "projects should be a list");

❌ Avoid: "projects should be 'a list'" (incorrect quoting)
❌ Avoid: "flag should be 'true'" (incorrect quoting for boolean)
```

### Format Strings and Placeholders

When including dynamic values in messages:

- Use `{}` for user-facing display values
- Use `{:?}` for debug output (developer-facing)
- Place placeholders at the **end** of the message
- Include context before the placeholder

```rust
✅ Good: format!("No init (conn {})", i)
✅ Good: format!("Failed to receive message from {}", worker_id)
✅ Good: format!("Failed to parse args: {:?}", args)

❌ Avoid: "no message" (missing context which connection)
❌ Avoid: "failed" (what failed?)
❌ Avoid: format!("Failed {} to read {}", path, error) (placeholders not at end)
```

### Multi-Line Messages

For complex conditions, use multi-line assertions with clear messages:

```rust
✅ Good: assert!(
    result.is_ok(),
    "Bead creation should succeed: {:?}",
    result.unwrap_err()
)

✅ Good: assert!(
    projects.is_array(), 
    "projects should be a list, got {:?}", 
    projects
)

❌ Avoid: assert!(result.is_ok()) // no context on failure
```

---

## Context Inclusion Guidelines

### File and Location Context

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

### Function and Operation Context

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

### Field and Property Context

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

### Component and Subsystem Context

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

### System State Context

**Include system state when:**
- The error is resource-related
- State affects operation success
- Debugging requires knowing current state

**✅ Include:**
```rust
"Failed to spawn worker: capacity full (3/5 slots in use)"
"Failed to acquire lock: held by operation {}"
"Connection pool exhausted (10/10 connections active)"
```

---

## Actionability Guidelines

### When to Provide Fix Suggestions

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

### How to Structure Suggestions

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

### Actionability Levels

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

### Audience-Based Actionability

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

### When Actionability is Appropriate vs Purely Informational

**Use Actionable Messages When:**

**✅ Use actionability for:**
- User-facing CLI errors
- Setup and initialization errors
- Configuration validation errors
- File permission errors
- Missing dependency errors
- Invalid input errors (with clear fix)

**Use Purely Informational Messages When:**

**✅ Use informational-only for:**
- Internal diagnostics and debugging
- Test assertion messages
- Invariant violations (developer-only)
- Complex errors requiring investigation
- Errors with multiple potential causes

**Mixed Approach (Informational + Diagnostic Hint):**

When errors are complex but have diagnostic value:

```rust
// ✅ Informational error with diagnostic hint
"Failed to parse bead line at offset 12345. Hint: Check that line format matches: <id>|<title>|<status>|...|"

// ✅ Informational with debug guidance
"Config validation failed. Hint: Run with --debug flag to see detailed validation errors"

// ✅ Informational with reference
"Reflection ledger consistency check failed. See docs/operations.md#reflection-ledger for recovery procedures"
```

---

## Error Type Standards

### `assert_eq!` - Equality Assertions

**Pattern:**
```rust
assert_eq!(actual, expected, "{what} should {state}")
```

**Examples:**
```rust
✅ Good: assert_eq!(resp.status(), 200, "healthz should return 200");
✅ Good: assert_eq!(parsed.subcommand, Some("scan".to_string()), 
                   "subcommand should be 'scan'");
✅ Good: assert_eq!(open_count, 2, "Should have 2 open beads");

❌ Avoid: assert_eq!(resp.status(), 200); // No context
```

### `assert!` - Boolean Assertions

**Pattern:**
```rust
assert!(condition, "{subject} should {state}[, context]")
```

**Examples:**
```rust
✅ Good: assert!(projects.is_array(), "projects should be a list");
✅ Good: assert!(resp.status().is_success(), "Bead creation should succeed");
✅ Good: assert!(received_init, "Should receive init event");
✅ Good: assert!(create_resp.status().is_success(), 
                "Bead creation should succeed for project: {}", project_name);

❌ Avoid: assert!(condition); // No context
```

### `expect()` - Result Expectation

**Pattern:**
```rust
some_operation.expect("{what} failed[, context]")
```

**Examples:**
```rust
✅ Good: String::from_utf8(output.stderr)
            .expect("Invalid UTF-8 in stderr");
✅ Good: init_msg.expect("Failed to receive init message");
✅ Good: init_msg.expect(&format!("No init (conn {})", i));

❌ Avoid: result.expect("failed"); // What failed?
❌ Avoid: result.expect("error");  // Useless message
```

### `unwrap()` - Panic on None/Err

**Standard:** **Avoid bare `.unwrap()` in production code.** In tests, prefer `.expect()` with context.

```rust
❌ Avoid: let msg = some_option.unwrap();
✅ Good:  let msg = some_option.expect("message should be present");

❌ Avoid: let config = parse_config(path).unwrap();
✅ Good:  let config = parse_config(path)
                       .expect(&format!("Failed to parse config from {}", path));
```

### HTTP Status Assertions

**Pattern:**
```rust
assert_eq!(resp.status(), {code}, "{endpoint} should return {status}")
```

**Examples:**
```rust
✅ Good: assert_eq!(resp.status(), 200, 
                   "GET /api/beads should return 200");
✅ Good: assert_eq!(resp.status(), 404, 
                   "Non-existent endpoint should return 404");
✅ Good: assert!(resp.status().is_success(), 
                "Bead creation should succeed");

❌ Avoid: assert_eq!(resp.status(), 200);
```

### Field Validation Assertions

**Pattern:**
```rust
assert!(err.is_some(), "{invalid} {field} should fail");
assert!(err.field.is_some(), "error should include field path");
```

**Examples:**
```rust
✅ Good: assert!(err.is_some(), "missing schema_version should fail");
✅ Good: assert!(err.is_some(), "invalid adapter value should fail");
✅ Good: assert!(err.field.is_some(), "error should include field path");

❌ Avoid: assert!(err.is_some()); // What should fail?
```

### Panic Messages

**Pattern:**
```rust
panic!("Expected [entity] [context]");
```

**Examples:**
```rust
✅ Good: panic!("Expected Scan command");
✅ Good: panic!("Expected Remove command at Level 2");
✅ Good: panic!("Expected Projects command at Level 1");
```

---

## Complete Examples

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

### Example 6: Complete Test Example

**✅ Good: Clear, actionable, context-rich**
```rust
#[tokio::test]
async fn bead_creation_flow() {
    let resp = client
        .post("/api/beads")
        .json(&bead_data)
        .send()
        .await
        .expect("Failed to send bead creation request");

    assert_eq!(
        resp.status(), 
        201, 
        "Bead creation should return 201 Created"
    );

    let body: Value = resp
        .json()
        .await
        .expect("Failed to parse bead creation response");

    assert!(!body["id"].as_str().unwrap().is_empty(), 
            "Created bead should have non-empty ID");
    
    let list_resp = client
        .get("/api/beads")
        .send()
        .await
        .expect("Failed to fetch bead list");

    assert_eq!(
        list_resp.status(), 
        200, 
        "GET /api/beads should return 200"
    );
}
```

**❌ Bad: Vague, missing context, unhelpful**
```rust
#[tokio::test]
async fn bead_creation_flow() {
    let resp = client.post("/api/beads").json(&bead_data).send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert!(!body["id"].as_str().unwrap().is_empty());
}
```

---

## Anti-Patterns to Avoid

### 1. Bare Assertions Without Messages

```rust
❌ Avoid: assert_eq!(resp.status(), 200);
❌ Avoid: assert!(condition);
❌ Avoid: assert!(result.is_ok());

✅ Fix: assert_eq!(resp.status(), 200, "healthz should return 200");
✅ Fix: assert!(condition, "connection should be established");
✅ Fix: assert!(result.is_ok(), "operation should succeed");
```

### 2. Generic or Useless Messages

```rust
❌ Avoid: result.expect("failed");
❌ Avoid: result.expect("error");
❌ Avoid: result.expect("panic");

✅ Fix: result.expect("Failed to parse bead from response");
✅ Fix: result.expect("Failed to connect to daemon at {}", socket_path);
✅ Fix: result.expect("Failed to deserialize config: invalid schema_version");
```

### 3. Missing Context

```rust
❌ Avoid: assert!(init_msg.is_some());
✅ Fix: assert!(init_msg.is_some(), 
               "Should receive init message from daemon");

❌ Avoid: assert_eq!(fetched["id"], original["id"]);
✅ Fix: assert_eq!(fetched["id"], original["id"], 
                   "Fetched bead ID should match original");
```

### 4. Inconsistent Wording

```rust
❌ Avoid: Mixed patterns in same test:
    assert_eq!(resp.status(), 200, "should be 200");
    assert!(body.is_ok(), "response must be ok");
    assert_eq!(count, 5, "expected 5 items");

✅ Fix: Consistent pattern:
    assert_eq!(resp.status(), 200, "healthz should return 200");
    assert!(body.is_ok(), "body should be present");
    assert_eq!(count, 5, "should have 5 items");
```

### 5. Bare Unwrap Without Context

```rust
❌ Avoid: let bead_id = response["id"].as_str().unwrap();
❌ Avoid: let config = parse_config(path).unwrap();

✅ Fix: let bead_id = response["id"]
            .as_str()
            .expect("bead ID should be present in response");
✅ Fix: let config = parse_config(path)
            .expect(&format!("Failed to parse config from {}", path));
```

### 6. Over-Actionable Messages

```rust
❌ Avoid speculating on fixes when the cause is unclear:
"Failed to read config. Maybe file permissions are wrong? Try reinstalling"

✅ Better - diagnostic:
"Failed to read config from: {}. Check file exists and is readable", path
```

### 7. Over-Verbose Messages

```rust
❌ Avoid overwhelming detail:
"Failed to read config file located at path /home/user/.hoop/config.yml with error code 2 indicating \
file not found which means the file does not exist at the specified location"

✅ Better - concise:
"Failed to read config from ~/.hoop/config.yml: file not found. Run 'hoop init' to create config"
```

### 8. Vague Action Suggestions

```rust
❌ Avoid non-specific suggestions:
"Config error. Check the config file"
"Failed to parse. Fix the error"

✅ Better - specific:
"Config validation failed at field 'schema_version': expected string, got integer"
"Failed to parse bead line: missing claimed_at field at line 15"
```

### 9. Missing Critical Context

```rust
❌ Never omit context that's needed for diagnosis:
"Failed to read config" // which config?
"got integer" // but what was expected?
"validation failed" // what failed validation?

✅ Always include:
"Failed to read config from ~/.hoop/config.yml"
"Expected string, got integer"
"schema_version validation failed: must be string"
```

### 10. Single-Word Messages

```rust
❌ Avoid: "scan", "/tmp", "--no-interactive" as assert messages

✅ Use descriptive messages:
"subcommand should be 'scan' when scan command is invoked"
"test project directory should be created at /tmp"
"--no-interactive flag should be parsed correctly"
```

---

## Quality Checklist

### Before Committing New Error Messages

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

**Format compliance:**
- [ ] Follows "should"/"Failed to"/"must" patterns
- [ ] Preserves original case for identifiers
- [ ] Uses `{:?}` for debug, `{}` for display
- [ ] No trailing period
- [ ] Placeholders at end of message

**Quoting and literals:**
- [ ] Quotes around string values and commands
- [ ] No quotes around types and booleans
- [ ] Consistent quoting patterns across similar errors

**Structure:**
- [ ] Subject-first order when applicable
- [ ] Expected vs. actual order in comparisons
- [ ] Clear cause-effect relationship in failure messages

---

## Usage in HOOP

These standards apply to:

- **Test assertions** - Developer-facing, self-documenting intent
- **Production error handling** - User-facing, actionable when possible
- **CLI error messages** - User-facing, clear guidance
- **API error responses** - Structured, machine-readable + human-readable
- **Panic/unwrap messages** - Invariant violations, debugging context

---

## Migration Path

### Phase 1: Add Minimum Context
Add what/where/expected to bare messages:
```rust
// Before
unwrap()

// After
expect("Failed to read config")
```

### Phase 2: Add Cause and Context
Include file paths, error details, system state:
```rust
// Before
expect("Failed to read config")

// After  
expect(&format!("Failed to read config from {}: {}", path, error))
```

### Phase 3: Add Actionable Suggestions
Include safe, obvious fixes where appropriate:
```rust
// Before
expect(&format!("Failed to read config from {}: {}", path, error))

// After
expect(&format!("Failed to read config from {}: file not found. Run 'hoop init' to create it", path))
```

---

## Related Resources

- **[Error Message Catalog](../error_messages_catalog.md)** - Current inventory of 5,904 error messages
- **[Pattern Analysis](../error_message_pattern_analysis.md)** - Detailed analysis of existing patterns
- **[AGENTS.md](../AGENTS.md)** - Repository guide for LLMs
- **[Plan](../docs/plan/plan.md)** - HOOP implementation plan

---

**Document Status:** Complete  
**Version:** 1.0  
**Last Updated:** 2026-08-12  
**Next Steps:** Apply these standards in error message improvement work across HOOP codebase
