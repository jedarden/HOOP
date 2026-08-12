# HOOP Error Message Informational and Actionability Standards

**Version:** 1.0  
**Created:** 2026-08-12  
**Scope:** HOOP test suite and production code across all crates  
**Task:** Define informational and actionability standards (bead bf-4ory0)  
**Related:** [Wording and Formatting Standards](error-message-standards.md)

## Purpose

This document defines comprehensive standards for what information error messages must include and how they guide users toward solutions. It complements the [Wording and Formatting Standards](error-message-standards.md) by providing deeper guidance on content requirements and actionability.

## Core Principles

Error messages should balance **thoroughness** with **verbosity** to maximize **developer experience** and **debugging efficiency**:

1. **Sufficient context** - Include enough information to locate and understand the failure
2. **Actionable guidance** - Tell the user what they can do next
3. **No redundancy** - Don't repeat information already visible in the stack trace or code
4. **Audience awareness** - Match detail level to the expected reader (developer vs operator)
5. **Cognitive load** - Prioritize information by importance; don't overwhelm with irrelevant details

---

## 1. Minimum Informational Content

### 1.1 Universal Minimum Requirements

Every error message MUST include:

#### For Assertions and Expectations:
1. **What was being tested** - The subject of the assertion
2. **What was expected** - The desired outcome
3. **Context** - Where the assertion is happening (function, test case, scenario)

#### For Operations and Parsing:
1. **What operation failed** - The action being attempted
2. **What was being operated on** - The input data, file, or resource
3. **Where it failed** - Location in the input (line number, field path, position)

#### For Configuration and Setup:
1. **What's missing or invalid** - The specific problem
2. **What was expected** - The correct format or value
3. **Where the problem is** - File path, config key, or setting name

### 1.2 Minimum Content Examples

#### ✅ GOOD: Meets minimum requirements
```rust
// Assertion with all required elements
assert!(
    events.contains("claim"),
    "Events fixture should contain claim event (found {} types)",
    event_types.len()
);

// Operation with clear context
expect("Failed to parse events.jsonl: invalid JSON at line 42");

// Configuration with specific problem
anyhow::bail!("Config file missing required 'agent' section at path/to/config.yml");
```

#### ❌ AVOID: Missing minimum content
```rust
// No context about what was being tested
assert!(events.contains("claim"), "events incomplete");

// No information about what failed or where
expect("Failed to parse file");

// No indication of what's missing
anyhow::bail!("Config error");
```

### 1.3 Content Requirements Matrix

| Error Type | What Failed | What Was Expected | Context | Location |
|------------|-------------|-------------------|---------|----------|
| **Assertion** | ✅ Required | ✅ Required | ✅ Required | Optional (in stack) |
| **File Operation** | ✅ Required | N/A | ✅ File path | ✅ Line/position if applicable |
| **Parsing** | ✅ Required | ✅ Valid format | ✅ Input snippet | ✅ Position/line number |
| **Config Validation** | ✅ Required | ✅ Valid value | ✅ Config key/path | ✅ Config file location |
| **Network/API** | ✅ Required | ✅ Success status | ✅ Endpoint/operation | ✅ URL/path if applicable |

---

## 2. When to Include Context

### 2.1 Context Inclusion Guidelines

Context should be included when it answers one of these questions:

- **Where** did this error occur? (file, function, test case)
- **On what** data or resource did it occur? (specific file, field, variable)
- **Under what conditions** did it occur? (configuration state, test setup)
- **Which** of many similar things failed? (array index, loop iteration, enum variant)

### 2.2 Function and Module Names

#### ✅ INCLUDE function names when:
- The error could originate from multiple locations
- The function name clarifies what operation failed
- The message appears in logs or output without stack traces

```rust
// GOOD: Function name clarifies the operation
expect("parse_events_jsonl: unexpected EOF at line 42");
anyhow::bail!("load_config: schema_version field missing");

// GOOD: Function name helps distinguish similar errors
assert!(
    validate_schema(&config).is_ok(),
    "validate_schema: config should pass schema validation"
);
```

#### ❌ OMIT function names when:
- They duplicate information already visible in stack traces
- They're generic names that don't add clarity (`main`, `run`, `execute`)
- The error is clearly about a specific operation or data

```rust
// AVOID: Function name is redundant with operation description
expect("read_file: failed to read file");  // "read_file" adds nothing

// AVOID: Generic function name
expect("run: command failed");  // "run" is too generic
```

### 2.3 File Locations and Paths

#### ✅ INCLUDE file paths when:
- The operation explicitly targets files (read, write, parse)
- Multiple files could be the source of the error
- The user needs to locate or inspect the file

```rust
// GOOD: Specific file path
expect("Failed to read ~/.hoop/config.yml: permission denied");
anyhow::bail!("projects.rs not found in workspace root: {}", workspace_path);

// GOOD: File path with additional context
anyhow::bail!(
    "testrepo/.beads/events.jsonl is empty or corrupted. \
     Run ./scripts/setup-testrepo.sh to regenerate fixtures"
);
```

#### ❌ OMIT file paths when:
- The file is implied by the operation (e.g., standard config locations)
- The path is extremely long and would clutter the message
- The file location is obvious from context

```rust
// AVOID: Extremely long paths that obscure the message
expect!("Failed to read /home/coding/very/long/path/that/obscures/message/file.txt");

// BETTER: Truncate or reference
expect!("Failed to read .../very/long/path/file.txt");
expect!("Failed to read workspace config.yml");
```

### 2.4 Line Numbers and Positions

#### ✅ INCLUDE line/position when:
- Parsing structured data (JSON, YAML, text files)
- Processing large inputs where failure location isn't obvious
- Validating multi-line or multi-field structures

```rust
// GOOD: Specific line number for parsing errors
anyhow::bail!("events.jsonl: invalid JSON at line 42: {}", error_msg);

// GOOD: Field path for config errors
assert!(
    err.field.as_ref().unwrap().contains("agent.adapter"),
    "error field should identify agent.adapter: {:?}",
    err.field
);

// GOOD: Position in data stream
anyhow::bail!("Invalid event at position {}: missing 'timestamp' field", i);
```

### 2.5 Loop Indices and Iterations

#### ✅ INCLUDE loop indices when:
- Processing arrays or collections where specific items fail
- The index helps identify which data is problematic
- Iterating over heterogeneous data

```rust
// GOOD: Index helps debug which item failed
for (i, event) in events.iter().enumerate() {
    assert!(
        !event.bead_id.is_empty(),
        "Event {}: bead_id should not be empty",
        i
    );
}

// GOOD: Index helps identify which command failed
for (i, cmd) in commands.iter().enumerate() {
    assert!(
        cmd.is_valid(),
        "Command {}: {:?} should be valid",
        i, cmd
    );
}
```

#### ❌ OMIT loop indices when:
- The loop processes a single item or very small collection
- The index doesn't help identify which specific data failed
- The failure is about the collection as a whole, not individual items

```rust
// AVOID: Index doesn't add value for single-item or obvious failures
for (i, item) in single_item_list.iter().enumerate() {
    assert!(item.is_valid(), "Item {} should be valid", i); // Unnecessary index
}

// BETTER: Focus on the actual problem
assert!(single_item.is_valid(), "Item should be valid");
```

### 2.6 Test Case and Scenario Names

#### ✅ INCLUDE test/scenario names when:
- Running parameterized tests with multiple cases
- Errors appear in aggregated test output
- Multiple similar tests could be confused

```rust
// GOOD: Test case name clarifies which scenario failed
#[test]
fn test_scan_parse_with_flag_before_subcommand() {
    let result = parse_cli_with_flag(&["hoop", "--no-interactive", "scan", "/tmp"]);
    assert!(result.is_ok(), "Should parse flag before subcommand");
}

// GOOD: Scenario parameter in message
for scenario in &["flag before", "flag after", "no flag"] {
    assert!(
        test_scenario(scenario).is_ok(),
        "Scenario '{}' should pass",
        scenario
    );
}
```

---

## 3. When and How to Provide Suggestions for Fixes

### 3.1 When Suggestions Are Appropriate

Provide fix suggestions when:

1. **The fix is standard or well-known** - Common configuration errors, missing dependencies
2. **The user might not know the solution** - Setup errors, permission issues
3. **There are multiple possible fixes** - Help the user choose the right one
4. **The error is likely to recur** - Save debugging time on future occurrences

### 3.2 When to Omit Suggestions

Don't provide suggestions when:

1. **The fix is obvious** - Simple syntax errors, typos
2. **The error is transient** - Network timeouts, temporary failures
3. **The suggestion would be speculative** - Don't guess at solutions
4. **The user is likely an expert** - Internal errors, implementation details

### 3.3 How to Structure Suggestions

#### Pattern 1: Direct solution (specific action)
```rust
// ✅ GOOD: Clear, actionable suggestion
anyhow::bail!(
    "Config file not found at {:?}. \
     Create a config.yml at ~/.hoop/ or set HOOP_CONFIG environment variable.",
    path
);

// ✅ GOOD: Specific command to run
anyhow::bail!(
    "testrepo fixtures not initialized. \
     Run './scripts/setup-testrepo.sh' to generate test fixtures."
);
```

#### Pattern 2: Multiple solutions (menu of options)
```rust
// ✅ GOOD: Multiple clear options
anyhow::bail!(
    "No projects found. Options:\n\
     1. Create a new project: hoop project create <name>\n\
     2. Import existing workspace: hoop project import <path>\n\
     3. Set HOOP_PROJECTS_DIR to your projects directory"
);
```

#### Pattern 3: Explanation then solution (educational)
```rust
// ✅ GOOD: Explains why then suggests how
anyhow::bail!(
    "Config file at {:?} uses deprecated 'agent.adapter' field. \
     This field was removed in v0.5.0. \
     Update config.yml to use 'agent.model' instead. \
     See migration guide: docs/migrations/v0.5.0.md",
    path
);
```

#### Pattern 4: Diagnostic information plus suggestion
```rust
// ✅ GOOD: Provides context then solution
assert!(
    err.field.is_some(),
    "Error should include field path for diagnostics. \
     Got error without field: {:?}. \
     Ensure error construction includes field context.",
    err
);
```

### 3.4 Suggestion Quality Guidelines

#### ✅ DO make suggestions:
- **Specific** - Name exact commands, file paths, or values
- **Tested** - Verify the suggestion actually works
- **Current** - Ensure commands and paths match the current codebase
- **Safe** - Don't suggest destructive operations

#### ❌ DON'T make suggestions:
- **Speculative** - Don't guess at potential solutions
- **Obsolete** - Update suggestions when code changes
- **Dangerous** - Avoid suggestions that could cause data loss
- **Overwhelming** - Limit to 2-3 clear options

### 3.3 Suggestion Examples by Category

#### Configuration Errors
```rust
// ✅ GOOD: Specific file location and example
anyhow::bail!(
    "Missing required 'agent' section in config.yml. \
     Add:\n\
     \n\
     agent:\n\
       adapter: claude\n\
       model: claude-opus-4-7\n\
     \n\
     to ~/.hoop/config.yml"
);

// ✅ GOOD: Validation with specific requirement
anyhow::bail!(
    "Invalid value for 'agent.rate_limit_rpm': {}. \
     Must be a positive integer between 1 and 1000.",
    value
);
```

#### File/Resource Errors
```rust
// ✅ GOOD: Clear fix with command
anyhow::bail!(
    "projects.rs not found in workspace. \
     Initialize workspace with: hoop init"
);

// ✅ GOOD: Permission issue with solution
anyhow::bail!(
    "Cannot write to {}: permission denied. \
     Check file ownership or run with appropriate permissions.",
    path
);
```

#### Dependency/Setup Errors
```rust
// ✅ GOOD: Missing dependency with install command
anyhow::bail!(
    "br command not found in PATH. \
     Install beads-rust: cargo install beads-rust"
);

// ✅ GOOD: Version requirement with check command
anyhow::bail!(
    "br version {} is incompatible. \
     HOOP requires beads-rust >= 0.3.0. \
     Check version: br --version",
    version
);
```

---

## 4. How to Structure Actionable Error Messages

### 4.1 Actionable Message Structure

An actionable error message follows this structure:

```
[What failed] + [Context] + [Why it failed] + [What to do]
```

#### Complete example:
```
Failed to parse config.yml (operation): 
invalid value for 'agent.rate_limit_rpm' (context): 
must be between 1 and 1000, got 1500 (why): 
Update config.yml to use a value in the valid range (what to do)
```

### 4.2 Actionability Levels

#### Level 1: Fully Actionable (tells user what to do)
```rust
// ✅ FULLY ACTIONABLE: Complete guidance
anyhow::bail!(
    "Config file not found at {:?}. \
     Create ~/.hoop/config.yml or set HOOP_CONFIG environment variable.",
    path
);

// Elements:
// - What failed: Config file not found
// - Where: Specific path
// - What to do: Create file or set env var
```

#### Level 2: Contextual Actionable (points to solution)
```rust
// ✅ CONTEXTUAL: Provides enough info to act
assert!(
    events.contains("claim"),
    "Events fixture should contain 'claim' event. \
     Check scripts/setup-testrepo.sh"
);

// Elements:
// - What failed: Missing event type
// - Context: Events fixture
// - Hint: Check setup script (not explicit command, but actionable)
```

#### Level 3: Diagnostic Actionable (enables debugging)
```rust
// ✅ DIAGNOSTIC: Enough info to investigate
assert!(
    err.field.is_some(),
    "Error should include field path: got {:?}",
    err
);

// Elements:
// - What failed: Missing field in error
// - What was found: Actual error value
// - Next step: Debug why field is missing
```

#### Level 4: Informational (describes problem without solution)
```rust
// ⚠️ INFORMATIONAL: States what's wrong, no guidance
anyhow::bail!("Config file uses deprecated 'agent.adapter' field");

// This is appropriate when:
// - The error is obvious or rare
// - The user is expected to know the fix
// - Providing a suggestion would be speculative
```

### 4.3 Structuring Multi-Line Messages

For complex errors, structure multi-line messages for readability:

```rust
// ✅ GOOD: Structured multi-line error
anyhow::bail!(
    "Config validation failed:\n\
     \n\
     Problems:\n\
     - Missing required field: agent.adapter\n\
     - Invalid value: agent.rate_limit_rpm = -1 (must be positive)\n\
     \n\
     Fix:\n\
     Update ~/.hoop/config.yml with valid values. \
     See config reference: docs/config.md\n\
     \n\
     Schema location: docs/schema/config.json"
);

// Structure:
// - Summary line
// - Specific problems (bulleted)
// - Fix section
// - Reference link
```

### 4.4 Actionability by Error Type

| Error Type | Minimum Actionability | Recommended Structure |
|------------|----------------------|----------------------|
| **Config validation** | Level 1 (fully actionable) | What's wrong + what value to use |
| **File not found** | Level 1 (fully actionable) | Path + how to create/obtain |
| **Permission denied** | Level 2 (contextual) | Path + suggestion to check permissions |
| **Parsing failure** | Level 3 (diagnostic) | Location + what was expected |
| **Assertion failure** | Level 3 (diagnostic) | What was checked + expected vs actual |
| **Network timeout** | Level 4 (informational) | What operation + timeout duration |
| **Internal error** | Level 4 (informational) | What failed + stack trace details |

---

## 5. When Actionability Is Appropriate vs. Informational Only

### 5.1 Use Fully Actionable Messages (Level 1) When:

- **User-facing operations** - Configuration, CLI commands, setup
- **Common mistakes** - Errors that are likely to recur
- **Standard problems** - Well-documented issues with known solutions
- **Prerequisite failures** - Missing dependencies, files, or setup

#### Examples:
```rust
// User-facing config error
anyhow::bail!(
    "Missing required field '{}'. \
     Add it to your config.yml. \
     Example: {} = \"{}\"",
    field_name, field_name, example_value
);

// Common mistake
anyhow::bail!(
    "Running HOOP in non-interactive mode without required --project flag. \
     Either:\n\
     1. Add --project <name> to specify the project\n\
     2. Omit --no-interactive to use interactive mode"
);

// Standard problem
anyhow::bail!(
    "br command not found. \
     Install with: cargo install beads-rust"
);
```

### 5.2 Use Contextual Actionable Messages (Level 2) When:

- **Complex setups** - Error depends on environment or state
- **Multiple possible causes** - One fix doesn't fit all scenarios
- **Expert users** - Users know the domain but need a hint

#### Examples:
```rust
// Complex setup (multiple possible issues)
anyhow::bail!(
    "Cannot connect to br daemon. \
     Check:\n\
     - br daemon is running (br daemon status)\n\
     - HOOP_DAEMON_ADDR is correct\n\
     - Network connectivity to daemon"
);

// Multiple possible causes
anyhow::bail!(
    "Failed to parse events.jsonl. \
     Ensure:\n\
     - File exists and is readable\n\
     - Format is NDJSON (one JSON object per line)\n\
     - Each line has required fields: event_type, bead_id"
);
```

### 5.3 Use Diagnostic Messages (Level 3) When:

- **Development debugging** - Internal errors for developers
- **Complex failures** - Root cause isn't immediately obvious
- **Assertion failures** - Test failures where context helps investigation

#### Examples:
```rust
// Development debugging
assert!(
    result.is_ok(),
    "Schema validation should pass. \
     Got error: {:?} \
     Schema: {:?} \
     Input: {:?}",
    result.err(),
    schema,
    input
);

// Complex failure
anyhow::bail!(
    "Context index inconsistent: \
     found {} entries but metadata claims {}. \
     Index may need rebuilding. \
     Run: hoop index rebuild",
    actual_count,
    metadata_count
);

// Assertion with investigation context
assert_eq!(
    parsed.version,
    expected_version,
    "Version mismatch in {}: \
     parsed {:?} but expected {:?} \
     Raw content: {:?}",
    file_path,
    parsed,
    expected,
    raw_content
);
```

### 5.4 Use Informational-Only Messages (Level 4) When:

- **Obvious failures** - The error is self-explanatory
- **Transient issues** - Network timeouts, temporary failures
- **Internal state** - Errors meant for logs/monitoring, not users
- **Expert context** - The audience knows how to investigate

#### Examples:
```rust
// Obvious failure
anyhow::bail!("Connection timeout after 30s");

// Transient issue
anyhow::bail!("Temporary network failure: connection reset");

// Internal state (for logs)
panic!("Invariant violated: event_count should never be negative");

// Expert context (developes know what this means)
anyhow::bail!("SQLite constraint violation: UNIQUE constraint failed: beads.id");
```

### 5.5 Decision Tree for Actionability Level

```
Is the error user-facing (config, CLI, setup)?
├─ Yes → Use Level 1 (Fully Actionable)
│         Provide specific steps or commands
└─ No → Is it a common or recurring mistake?
    ├─ Yes → Use Level 1 or Level 2
    │         Provide solution or hint
    └─ No → Is it for development/debugging?
        ├─ Yes → Use Level 3 (Diagnostic)
        │         Include investigation context
        └─ No → Is it obvious or transient?
            ├─ Yes → Use Level 4 (Informational)
            │         State the problem simply
            └─ No → Use Level 2 (Contextual)
                    Provide guidance without explicit commands
```

---

## 6. Positive and Negative Examples

### 6.1 Minimum Informational Content

#### ✅ POSITIVE: Meets all requirements
```rust
// Includes: what failed, what was expected, context
assert!(
    events.contains("claim"),
    "Events fixture should contain 'claim' event \
     (found {} types: {:?})",
    event_types.len(),
    event_types
);

// Includes: operation, target, location
expect(
    "Failed to parse config.yml at line 42: \
     invalid value for 'agent.adapter'"
);
```

#### ❌ NEGATIVE: Missing required information
```rust
// No context about what was being tested
assert!(events.contains("claim"), "incomplete");

// No indication of what failed or where
expect("Failed to parse");

// Missing what's expected vs actual
assert_eq!(value, expected);
```

### 6.2 Context Inclusion

#### ✅ POSITIVE: Appropriate context
```rust
// Function name clarifies operation
expect("load_projects: failed to read projects.rs");

// File path specific to error
anyhow::bail!("~/.hoop/config.yml: missing 'agent' section");

// Line number for parsing error
anyhow::bail!("events.jsonl:42: invalid JSON syntax");

// Loop index identifies failing item
for (i, event) in events.iter().enumerate() {
    assert!(
        event.bead_id.is_valid(),
        "Event {}: bead_id should be valid",
        i
    );
}
```

#### ❌ NEGATIVE: Missing or inappropriate context
```rust
// Function name adds nothing
expect("read_file: failed to read file");

// Path is implied or redundant
anyhow::bail!("Failed to read config file: ~/.hoop/config.yml is missing");

// Missing index in loop
for event in events {
    assert!(event.is_valid(), "Event should be valid");
}

// Obvious from stack trace
assert!(condition, "test_function: assertion failed");
```

### 6.3 Fix Suggestions

#### ✅ POSITIVE: Actionable and appropriate
```rust
// Config error with specific guidance
anyhow::bail!(
    "Missing required 'agent.adapter' field. \
     Add to config.yml:\n\
     agent:\n\
       adapter: claude"
);

// Setup error with command
anyhow::bail!(
    "Test fixtures not initialized. \
     Run: ./scripts/setup-testrepo.sh"
);

// Multiple clear options
anyhow::bail!(
    "No projects found. Options:\n\
     1. Create new: hoop project create <name>\n\
     2. Import existing: hoop project import <path>"
);
```

#### ❌ NEGATIVE: Missing, speculative, or overwhelming
```rust
// No guidance (common mistake with no hint)
anyhow::bail!("Missing 'agent' field");

// Speculative suggestion
anyhow::bail!("Config error. Try reinstalling HOOP");

// Too many options (overwhelming)
anyhow::bail!(
    "Config error. Possible fixes:\n\
     1. Check YAML syntax\n\
     2. Verify file permissions\n\
     3. Reinstall HOOP\n\
     4. Check disk space\n\
     5. Run diagnostics\n\
     6. Contact support..."
);

// Dangerous suggestion
anyhow::bail!("Permission denied. Run: sudo rm -rf ~/.hoop");
```

### 6.4 Actionable vs Informational

#### ✅ POSITIVE: Appropriate actionability level
```rust
// User-facing → Fully actionable
anyhow::bail!(
    "Config not found. Create ~/.hoop/config.yml \
     or set HOOP_CONFIG environment variable"
);

// Development → Diagnostic
assert!(
    validate_schema(&input).is_ok(),
    "Schema validation failed for input: {:?}. \
     Expected format: {:?}",
    input,
    expected_format
);

// Obvious/transient → Informational
anyhow::bail!("Connection timeout after 30s");
```

#### ❌ NEGATIVE: Mismatched actionability
```rust
// User-facing but no guidance
anyhow::bail!("Config validation failed");

// Internal error with excessive detail for users
anyhow::bail!(
    "Internal error: \
     SQLiteConstraintViolation at src/db.rs:42:\
     UNIQUE constraint failed on beads.id \
     during transaction_id=abc123 \
     with isolation_level=serializable"
);

// Simple issue with excessive guidance
anyhow::bail!(
    "File not found. \
     This error occurs when the specified file \
     does not exist at the given path. \
     To resolve:\n\
     1. Check the file path for typos\n\
     2. Verify the file exists\n\
     3. Ensure you have read permissions\n\
     ...\n\
     [10 more lines of explanation]"
);
```

---

## 7. Special Considerations

### 7.1 Developer Experience vs Verbosity

Balancing detail level with readability:

```rust
// ❌ TOO VERBOSE: Overwhelming detail
assert!(
    parsed_events.len() == 50,
    "During the execution of the parse_events_jsonl function \
     on the file located at the path testrepo/.beads/events.jsonl \
     which contains fixture data for testing purposes, \
     we expected to find exactly 50 events in the parsed result \
     based on the known fixture data, but instead we found {} events \
     which represents a discrepancy that indicates either the fixture \
     file has been modified or the parsing logic has a bug",
    parsed_events.len()
);

// ✅ BETTER: Concise but informative
assert!(
    parsed_events.len() == 50,
    "parse_events_jsonl: expected 50 events, got {}. \
     Fixture file: testrepo/.beads/events.jsonl",
    parsed_events.len()
);

// ✅ OPTIMAL: If this is in a test named test_parse_events_jsonl
assert_eq!(
    parsed_events.len(),
    50,
    "Expected 50 events from fixture, got {}",
    parsed_events.len()
);
```

### 7.2 Debugging Efficiency

Prioritize information by debugging usefulness:

```rust
// ✅ HIGH DEBUGGING VALUE: Structured context
anyhow::bail!(
    "Config validation failed:\n\
     Field: agent.adapter\n\
     Value: {:?}\n\
     Error: must be one of: claude, anthropic, zai\n\
     Config file: {}",
    actual_value,
    config_path
);

// ✅ HIGH DEBUGGING VALUE: Expected vs actual
assert_eq!(
    parsed.bead_id,
    expected_id,
    "Event {}: bead_id mismatch\n\
     Expected: {:?}\n\
     Got: {:?}\n\
     Raw event: {:?}",
    i,
    expected_id,
    parsed.bead_id,
    raw_events[i]
);

// ❌ LOW DEBUGGING VALUE: No context
anyhow::bail!("Config error");
```

### 7.3 Audience Awareness

Match detail level to expected audience:

```rust
// For operators (user-facing CLI)
// ✅ Appropriate: Clear, actionable
anyhow::bail!(
    "Config file at {:?} is invalid. \
     Run 'hoop config validate' for details.",
    path
);

// For developers (test assertions)
// ✅ Appropriate: Technical, diagnostic
assert!(
    matches!(err, Error::Validation { .. }),
    "Error should be Validation variant, got {:?}",
    err
);

// For logs (monitoring)
// ✅ Appropriate: Structured, machine-readable
error!(
    error = %err,
    config_path = %path,
    "Config validation failed"
);
```

---

## 8. Integration with Wording and Formatting Standards

This document complements the [Wording and Formatting Standards](error-message-standards.md). Use both documents together:

1. **Wording and Formatting** - How to phrase and style messages
2. **Informational and Actionability** - What content to include

### Quick Integration Guide

When writing an error message:

1. **Decide the actionability level** (Section 5)
2. **Include minimum required content** (Section 1)
3. **Add appropriate context** (Section 2)
4. **Provide fix suggestions if applicable** (Section 3)
5. **Structure for readability** (Section 4)
6. **Apply wording conventions** (Wording and Formatting Standards)

### Example Workflow

```rust
// Step 1: Decide actionability
// User-facing config error → Level 1 (Fully Actionable)

// Step 2: Include minimum content
// What failed: Config validation
// What's wrong: Missing required field
// Where: agent.adapter

// Step 3: Add context
// Config file location
// Expected value format

// Step 4: Provide fix suggestion
// Show example of correct config

// Step 5: Structure for readability
anyhow::bail!(
    "Config validation failed:\n\
     Missing required field: agent.adapter\n\
     Config file: {}\n\
     \n\
     Fix: Add to your config.yml:\n\
     agent:\n\
       adapter: claude  # or: anthropic, zai",
    path
);

// Step 6: Apply wording conventions
// - Sentence case
// - Periods at end of complete sentences
// - Backticks for code elements
```

---

## 9. Validation Checklist

Before considering an error message complete, verify:

### Informational Content
- [ ] **What failed** is clearly stated
- [ ] **What was expected** is specified (if applicable)
- [ ] **Context** is sufficient to locate the problem
- [ ] **Location** is included for file/parse errors

### Context Appropriateness
- [ ] Function names clarify (not duplicate) the operation
- [ ] File paths are specific and necessary
- [ ] Line/position numbers are included for parse errors
- [ ] Loop indices identify failing items

### Actionability
- [ ] Actionability level matches the audience (Section 5)
- [ ] Fix suggestions are specific and tested (if applicable)
- [ ] Suggestions aren't speculative or dangerous
- [ ] Message isn't overwhelming (2-3 suggestions max)

### Structure and Readability
- [ ] Message follows clear structure (what/context/why/fix)
- [ ] Multi-line messages are well-formatted
- [ ] Technical terms use correct formatting (backticks, quotes)
- [ ] Punctuation and capitalization follow standards

---

## 10. Appendix: Quick Reference

### Actionability Level Decision Guide

| Situation | Use Level | Example |
|-----------|-----------|---------|
| User-facing config/setup error | 1 (Fully Actionable) | "Config not found. Create ~/.hoop/config.yml or set HOOP_CONFIG" |
| Common mistake with known solution | 1 (Fully Actionable) | "br not found. Install with: cargo install beads-rust" |
| Complex setup with multiple checks | 2 (Contextual) | "Cannot connect. Check daemon status, HOOP_DAEMON_ADDR, and network" |
| Development assertion failure | 3 (Diagnostic) | "Expected validation success, got error: {:?}. Input: {:?}" |
| Obvious/transient issue | 4 (Informational) | "Connection timeout after 30s" |

### Minimum Content by Error Type

| Error Type | Minimum Required Elements |
|-------------|---------------------------|
| **Assertion** | What's tested, expected outcome, context |
| **File operation** | Operation, file path, error type |
| **Parsing** | What failed, where (line/position), what was expected |
| **Config validation** | What's invalid, correct format/value, config location |
| **Test failure** | What condition failed, expected vs actual, relevant variables |

### Context Inclusion Rules

| Context Type | Include When... | Omit When... |
|--------------|-----------------|---------------|
| Function name | Multiple call sites, name clarifies operation | Duplicates operation, generic name |
| File path | File-specific error, multiple possible files | Path implied, extremely long |
| Line number | Parse errors, large inputs | Not applicable to error type |
| Loop index | Collection iteration, index identifies item | Single item, index doesn't help |
| Test name | Parameterized tests, aggregated output | Test name obvious from context |

---

**Document Owner:** HOOP project maintainers  
**Related Documents:** [Wording and Formatting Standards](error-message-standards.md)  
**Last Reviewed:** 2026-08-12  
**Next Review:** 2026-09-12 or after validation of 100+ error messages
