# HOOP Error Message Standards

**Purpose:** Define consistent patterns for error messages across the HOOP test suite to ensure clarity, actionability, and maintainability.

**Generated:** 2026-08-12  
**Derived from:** Analysis of 5,904 error message patterns across 104 test files  
**Reference:** `error_messages_catalog.md` (bead bf-3ysoc)

---

## Table of Contents

1. [Principles](#principles)
2. [Wording Conventions](#wording-conventions)
3. [Formatting Patterns](#formatting-patterns)
4. [Informational Requirements](#informational-requirements)
5. [Actionability Guidelines](#actionability-guidelines)
6. [Error Type Standards](#error-type-standards)
7. [Examples](#examples)
8. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

---

## Principles

### 1. **Clarity First**
Error messages must be immediately understandable to developers encountering the test failure, even without deep context of the test suite.

### 2. **Actionability**
Whenever possible, messages should suggest what went wrong or what the expected state should be.

### 3. **Context-Rich**
Include relevant identifiers, field paths, or state descriptions to make failures debuggable without stepping through code.

### 4. **Consistency**
Follow the same patterns across similar assertion types to build predictable, scannable error output.

---

## Wording Conventions

### Standard Phrasing

| Pattern | Standard Form | Example |
|---------|--------------|---------|
| Expected behavior | `"Should {verb} {noun}"` | `"Should have 2 open beads"` |
| Negative expectation | `"Should not {verb}"` | `"Should not accept invalid input"` |
| HTTP status | `"{endpoint} should return {status}"` | `"healthz should return 200"` |
| Field validation | `"{field} should fail {condition}"` | `"missing schema_version should fail"` |
| Type checking | `"{noun} should be {type}"` | `"projects should be a list"` |
| Required field | `"{field} is required"` | `"error should include field path"` |

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

```
✅ Good: "projects should be a list"
✅ Good: "Daemon should be healthy"
✅ Good: "metrics should contain at least one valid metric line"

❌ Avoid: "Projects should be a List" (random capitalization)
❌ Avoid: "should have 2 open beads." (trailing period)
❌ Avoid: "metrics should contain valid metric line" (missing articles)
```

### Comparison Messaging

When comparing values, use **expected vs. actual** order consistently:

```
✅ Good: "expected 200 OK, got 404 Not Found"
✅ Good: "expected flag to be true, found false"
✅ Good: "expected 'scan', got 'status'"

❌ Avoid: "got 404 instead of 200" (inconsistent order)
❌ Avoid: "found false but expected true" (reversed order)
```

---

## Formatting Patterns

### Quoting and Literals

**Use quotes for:**
- String values: `expected "scan", got "status"`
- Field names: `"no_interactive" flag should be true`
- Literal values: `expected value 42, got null`

**Don't use quotes for:**
- Types: `should be a list`, `should be an object`
- Booleans: `flag should be true` (not `flag should be "true"`)
- Numbers: `should have 2 open beads`

```rust
✅ Good: assert_eq!(parsed.subcommand, Some("scan".to_string()), 
                   "expected 'scan', got something else");
✅ Good: assert!(projects.is_array(), "projects should be a list");

❌ Avoid: "projects should be 'a list'" (incorrect quoting)
❌ Avoid: "flag should be 'true'" (incorrect quoting for boolean)
```

### Format Strings and Context

When including dynamic values in messages:

```rust
✅ Good: format!("No init (conn {})", i)
✅ Good: format!("Failed to receive message from {}", worker_id)

❌ Avoid: "no message" (missing context which connection)
❌ Avoid: "failed" (what failed?)
```

### Multi-Line Messages

For complex conditions, use multi-line assertions with clear messages:

```rust
✅ Good: assert!(
    result.is_ok(),
    "Bead creation should succeed: {:?}",
    result.unwrap_err()
)

❌ Avoid: assert!(result.is_ok()) // no context on failure
```

---

## Informational Requirements

### Minimum Information per Error Type

| Error Type | Required Information | Optional but Recommended |
|------------|---------------------|--------------------------|
| `assert_eq!` | Left value, right value, meaning | Context about what's being compared |
| `assert!` | Condition being tested, why it matters | Expected state, relevant identifiers |
| `expect()` | What operation failed, what was expected | Context (thread ID, connection ID, etc.) |
| `unwrap_err()` | What error was expected | Context about the test scenario |
| HTTP assertions | Endpoint, expected status, actual status | Request body, relevant headers |
| Field validation | Field path, expected type/constraint | Actual value received |

### Context Requirements

**Always include when relevant:**
- **Identifiers:** Bead IDs, project names, worker names
- **Field paths:** `"agent.adapter"`, `"metrics.enabled"`
- **HTTP endpoints:** `"GET /api/beads"`, `"/healthz"`
- **Connection/Thread context:** `"(conn {})"`, `"(worker {})`

```rust
✅ Good: assert_eq!(fetched_bead["id"], bead["id"], 
                   "Fetched bead ID should match")

✅ Good: assert!(resp.status() == 404, 
                "Non-existent bead {} should return 404", bead_id)

❌ Avoid: assert_eq!(fetched_bead["id"], bead["id"]) 
// Which bead? What field?
```

### Type Information

When type checking fails, explain both expected and actual types:

```rust
✅ Good: assert!(projects.is_array(), 
                "projects should be a list, got {:?}", projects)

✅ Good: assert!(capacity.is_object() || capacity.is_array(),
                "Capacity should be object or array, got type {:?}", 
                capacity)

❌ Avoid: assert!(projects.is_array()) 
// No explanation of what type was received
```

---

## Actionability Guidelines

### Suggesting the Expected State

Messages should describe what the correct state should be:

```rust
✅ Good: "healthz should return 200"
✅ Good: "All WebSocket connections should receive init"
✅ Good: "Metrics should contain at least one valid metric line"
✅ Good: "New bead should appear in list"

❌ Avoid: "healthz check failed"
❌ Avoid: "WebSocket test failed"
❌ Avoid: "Metrics check failed"
```

### Explaining Why

For non-obvious assertions, explain the reasoning:

```rust
✅ Good: "Fetched bead ID should match" (data consistency)
✅ Good: "First message should be init event" (protocol requirement)
✅ Good: "Daemon should still be healthy after malformed messages" (robustness)

❌ Avoid: "IDs match" (why does this matter?)
❌ Avoid: "Correct message type" (which type? why?)
```

### Providing Next Steps (When Appropriate)

For validation errors, suggest what to fix:

```rust
✅ Good: "error should include field path for debugging"
✅ Good: "Invalid adapter value should fail with clear message"
✅ Good: "unknown field should be rejected"

// Consider including in message:
✅ Better: "unknown field 'extra_field' should be rejected (check schema)"
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

---

## Examples

### Complete Test Example

```rust
// ✅ Good: Clear, actionable, context-rich
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

// ❌ Bad: Vague, missing context, unhelpful
#[tokio::test]
async fn bead_creation_flow() {
    let resp = client.post("/api/beads").json(&bead_data).send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert!(!body["id"].as_str().unwrap().is_empty());
}
```

### Validation Test Example

```rust
// ✅ Good: Clear validation messages
#[test]
fn test_config_validation() {
    let result = validate_config(invalid_yaml);
    
    assert!(result.is_err(), "Invalid config should fail validation");
    
    let err = result.unwrap_err();
    assert!(err.field.is_some(), "error should include field path");
    assert!(
        err.field.unwrap().starts_with("agent."),
        "error should be in agent section"
    );
    assert!(
        err.message.contains("adapter"),
        "error message should mention 'adapter' field"
    );
}

// ❌ Bad: Cryptic validation
#[test]
fn test_config_validation() {
    let result = validate_config(invalid_yaml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.field.is_some());
    assert!(err.field.unwrap().starts_with("agent."));
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

---

## Implementation Checklist

When writing new tests or updating existing ones, ensure:

- [ ] Every `assert_eq!`, `assert!`, `expect()` has a descriptive message
- [ ] Messages follow standard wording conventions ("should {verb}")
- [ ] Context is included (identifiers, field paths, endpoints)
- [ ] Messages are sentence case with no trailing period
- [ ] Quotes are used correctly (strings/field names, not types/booleans)
- [ ] Bare `.unwrap()` is replaced with `.expect()` or proper error handling
- [ ] HTTP assertions include endpoint and expected status
- [ ] Field validation explains what should fail and why

---

## Validation and Enforcement

### Automated Checks

Consider adding lints or checks for:
1. Bare `unwrap()` calls in test code
2. Missing messages on `assert!`, `assert_eq!`, `expect()`
3. Message patterns that violate standards

### Code Review Checklist

When reviewing test code:
1. Are all assertions descriptive?
2. Do messages follow the wording conventions?
3. Is sufficient context provided for debugging?
4. Are HTTP responses clearly described?
5. Are validation errors actionable?

---

## References

- **Source Analysis:** `error_messages_catalog.md` (5,904 patterns across 104 files)
- **Industry Best Practices:** Rust testing guidelines, error message usability research
- **HOOP Context:** `AGENTS.md`, `docs/plan/plan.md`

---

## Version History

- **v1.0** (2026-08-12): Initial standards derived from error message catalog analysis

---

**Note:** These standards are derived from existing patterns in the HOOP codebase and industry best practices. They should be applied to all new test code and used as a guide for improving existing tests incrementally.
