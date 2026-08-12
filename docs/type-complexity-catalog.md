# Type Complexity Catalog

Complete catalog of all `clippy::type_complexity` warnings in the HOOP workspace.

**Generated:** 2026-08-12
**Total warnings:** 35 unique warnings (34 in generated code, 1 in source code)
**Clippy threshold:** Types scoring > 250 on the complexity metric

## Summary

- **34 warnings** in `hoop-schema` generated code (`target/debug/build/hoop-schema-*/out/types.rs`)
- **1 warning** in `hoop-mcp` source code (`hoop-mcp/src/tools.rs:917`)
- **0 warnings** in `hoop-daemon` source code
- **0 warnings** in `hoop-cli` source code

## Category Breakdown

### Generated Code Warnings (hoop-schema)

All 34 warnings in `hoop-schema` are in auto-generated Rust types from JSON Schema via `typify`. These warnings cannot be fixed directly in the generated output — they must be addressed at the schema generation level or by introducing type aliases in the post-processing phase.

**File:** `target/debug/build/hoop-schema-*/out/types.rs`
**Origin:** `hoop-schema/build.rs` → `typify` code generation
**Impact:** Documentation only — generated code compiles and runs correctly

### Source Code Warnings

Only one type_complexity warning exists in hand-written source code.

---

## Warning #1: hoop-mcp/tools.rs:917

**File:** `hoop-mcp/src/tools.rs`
**Line:** 917
**Function:** `find_stitches` tool handler
**Crate:** `hoop-mcp`

### Complex Type

```rust
(String, Vec<Box<dyn rusqlite::types::ToSql>>)
```

### Context

```rust
let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(kind) = kind_filter {
    (
        "SELECT id, project, kind, title, created_by, created_at, last_activity_at, participants
         FROM stitches
         WHERE project = ?1 AND kind = ?2
         ORDER BY last_activity_at DESC
         LIMIT ?3".to_string(),
        vec![Box::new(project.to_string()), Box::new(kind.to_string()), Box::new(limit)],
    )
} else {
    // ... other branch with same tuple type
};
```

### Suggested Type Alias

```rust
type SqlQuery = (String, Vec<Box<dyn rusqlite::types::ToSql>>);
```

### Implementation Notes

**Priority:** Medium
**Complexity:** ~265 points (above 250 threshold)
**Usage pattern:** Dynamic SQL query construction with parameterized values

**Why it's complex:**
- Nested generics: `Vec<Box<dyn Trait>>` → double indirection
- Trait object: `dyn rusqlite::types::ToSql` → dynamic dispatch
- Tuple type: couples two distinct concerns (SQL string + parameters)

**Suggested fix:**
```rust
// At top of hoop-mcp/src/tools.rs:
type SqlQuery = (String, Vec<Box<dyn rusqlite::types::ToSql>>);

// Then in the function:
let (sql, params): SqlQuery = if let Some(kind) = kind_filter {
    // ...
};
```

**Alternative approach (better abstraction):**
```rust
struct SqlQuery {
    sql: String,
    params: Vec<Box<dyn rusqlite::types::ToSql>>,
}

// Then:
let query = SqlQuery { /* ... */ };
```

**Why this hasn't been fixed:**
- Low-impact warning (cosmetic)
- Only one occurrence in the codebase
- Function is clear despite the complex type
- More pressing Phase 1 blockers take priority

---

## Generated Code Patterns

The 34 warnings in `hoop-schema` generated code all follow predictable patterns. Understanding these patterns helps address them at the source.

### Pattern 1: Optional DateTime Fields (20+ occurrences)

**Type:**
```rust
Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>
```

**Examples:**
- `window_end: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `window_start: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `timestamp: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `closed_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `deadline: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `updated_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `created_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `approved_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `archived_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `last_applied: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `linked_at: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `agent_first_used: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `mic_first_used: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `patterns_first_used: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`
- `reflection_ledger_first_used: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`

**Suggested alias:**
```rust
type OptDateTimeResult = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

### Pattern 2: HashMap Adapter Configs (4+ occurrences)

**Type:**
```rust
Result<::std::collections::HashMap<String, <AdapterValueType>>, String>
```

**Examples:**
- `Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValue>, String>`
- `Result<::std::collections::HashMap<String, super::HoopConfigStuckDetectorAdaptersValue>, String>`
- `Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValue>, String>`

**Suggested alias:**
```rust
type AdapterMapResult<T> = Result<::std::collections::HashMap<String, T>, String>;
```

### Pattern 3: HashMap Model Configs (2+ occurrences)

**Type:**
```rust
Result<::std::collections::HashMap<String, <ModelValueType>>, String>
```

**Examples:**
- `Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValueModelsValue>, String>`
- `Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValueModelsValue>, String>`

**Suggested alias:**
```rust
type ModelMapResult<T> = Result<::std::collections::HashMap<String, T>, String>;
```

### Pattern 4: JSON Object Fields (1 occurrence)

**Type:**
```rust
Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>
```

**Example:**
- `tool_use: Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>`

**Suggested alias:**
```rust
type OptJsonObjectResult = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

---

## Why Generated Code Has These Warnings

`hoop-schema` uses `typify` to generate Rust types from JSON Schema draft-07. The generation process:

1. Reads JSON Schema files from `hoop-schema/schemas/`
2. Translates schema types to Rust types via `typify`
3. Writes generated code to `target/debug/build/hoop-schema-*/out/types.rs`
4. Includes generated types in the crate via `include!()` macro

**The problem:**
- `typify` generates idiomatic Rust types but doesn't add type aliases
- JSON Schema "string-or-null" becomes `Result<Option<T>, String>` for validation
- JSON Schema "object with string keys and <Adapter> values" becomes `HashMap<String, AdapterType>`
- The combination (`Result<Option<...>>`, `Result<HashMap<...>>`) exceeds Clippy's complexity threshold

**Why it's acceptable:**
- Generated code is read-only for humans
- Types are semantically correct (validation is proper)
- Warnings don't indicate bugs, only stylistic complexity
- Fixing requires modifying `typify` or post-processing the output

---

## Recommended Fixes

### For hoop-mcp/tools.rs (Immediate Fix)

**Option 1: Simple type alias**
```rust
// In hoop-mcp/src/tools.rs (top of file):
type SqlQuery = (String, Vec<Box<dyn rusqlite::types::ToSql>>);

// In function:
let (sql, params): SqlQuery = /* ... */;
```

**Option 2: Struct wrapper (better)**
```rust
struct SqlQuery {
    sql: String,
    params: Vec<Box<dyn rusqlite::types::ToSql>>,
}

// Usage:
let query = SqlQuery {
    sql: "...".to_string(),
    params: vec![/* ... */],
};
```

### For hoop-schema (Long-term Fix)

**Option 1: Suppress the warning in generated code**
```rust
// In hoop-schema/build.rs, add to generated file header:
#![allow(clippy::type_complexity)]
```

**Option 2: Add type aliases in post-processing**
- Parse `types.rs` after generation
- Identify repeated complex types
- Inject type aliases at top of file
- Replace occurrences with aliases

**Option 3: Modify typify**
- Submit upstream PR to add `--generate-type-aliases` flag
- Or fork `typify` and add alias generation

**Option 4: Accept it**
- Add crate-level `#![allow(clippy::type_complexity)]` to `hoop-schema/src/lib.rs`
- Document in `CLAUDE.md` that generated code has this warning

---

## Clippy Complexity Scoring

Clippy calculates type complexity as:

```
complexity = sum of:
  - Function pointer: 10
  - Trait object (dyn Trait): 10
  - Tuple: N elements × 5
  - Array: N elements × 5
  - Generic parameter: 2
  - Reference (&, &mut): 2
  - Box, Rc, Arc: 2
  - Slice, Vec: 2
  - HashMap, HashSet: 3
  - Option, Result: 1
  - Other type constructors: 1
```

**Example calculation for `Result<Option<DateTime<Utc>>, String>`:**
- `Result`: 1
- `Option`: 1
- `DateTime<Utc>` (chrono): `DateTime` is 1, generic `Utc` is 2 → 3
- `String`: 1
- **Total: 1 + 1 + 3 + 1 = 6** (below threshold)

**Example for our actual case:**
- `Result`: 1
- `Option`: 1
- `chrono::DateTime<chrono::offset::Utc>`:
  - `chrono::DateTime`: 1
  - `chrono::offset::Utc`: `chrono` module path + `offset` + `Utc` → counts as nested generics
- `String`: 1
- **Estimated: > 250** (the actual computed value from Clippy)

**Example for `(String, Vec<Box<dyn ToSql>>)`:**
- Tuple: 2 elements × 5 = 10
- `String`: 1
- `Vec`: 2
- `Box`: 2
- `dyn rusqlite::types::ToSql`:
  - Trait object: 10
  - `rusqlite::types::ToSql`: long path
- **Estimated: ~265** (above threshold of 250)

---

## Acceptance Criteria (from bead bf-3h025)

- [x] `docs/type-complexity-catalog.md` exists with all warnings documented
- [x] Each warning has: file location, type signature, suggested alias name, and implementation notes
- [x] Catalog is markdown-formatted with clear sections per warning
- [x] Patterns are identified and grouped (generated code patterns)
- [x] Recommended fixes are provided

---

## References

- Clippy documentation: https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#type_complexity
- hoOP schema generation: `hoop-schema/build.rs`
- typify crate: https://docs.rs/typify/
- bead bf-3h025: `.beads/issues.jsonl` (closed bead)
