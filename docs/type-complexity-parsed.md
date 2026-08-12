# Type Complexity Warnings - Parsed and Categorized

## Overview

This document catalogs all `type_complexity` warnings from clippy, extracted from the raw clippy output. Total: **36 warnings** across 2 crates.

## Summary by Crate

| Crate | Warning Count | Source |
|-------|--------------|--------|
| `hoop-schema` | 35 | Generated code (`types.rs` from build script) |
| `hoop-mcp` | 1 | `tools.rs` |

## Pattern Categories

### Pattern A: Result-wrapped Optional DateTime (24 warnings)

**Pattern:** `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>`

**Description:** Fields that parse optional ISO timestamps from JSON, where `Ok(Some(T))` is a valid datetime, `Ok(None)` is a missing/null field, and `Err(String)` is a parse error.

**Occurrences:**

| Line | Field | Context |
|------|-------|---------|
| 22949 | `window_end` | DateTime window field |
| 22950 | `window_start` | DateTime window field |
| 24115 | `timestamp` | Generic timestamp field |
| 25066 | `last_success_iso` | Last success timestamp |
| 28206 | `timestamp` | Generic timestamp field |
| 28465 | `closed_at` | Closure timestamp |
| 28467 | `deadline` | Deadline timestamp |
| 28478 | `updated_at` | Update timestamp |
| 28692 | `added_at` | Addition timestamp |
| 28787 | `created_at` | Creation timestamp |
| 29763 | `approved_at` | Approval timestamp |
| 29765 | `archived_at` | Archive timestamp |
| 29769 | `last_applied` | Last application timestamp |
| 30409 | `timestamp` | Generic timestamp field |
| 30576 | `archived_at` | Archive timestamp |
| 30579 | `closed_at` | Closure timestamp |
| 30590 | `updated_at` | Update timestamp |
| 30846 | `linked_at` | Link timestamp |
| 30970 | `created_at` | Creation timestamp |
| 31597 | `end` | End datetime |
| 31598 | `start` | Start datetime |
| 32848 | `agent_first_used` | First agent use timestamp |
| 32849 | `mic_first_used` | First mic use timestamp |
| 32850 | `patterns_first_used` | First pattern use timestamp |
| 32851 | `reflection_ledger_first_used` | First reflection ledger use timestamp |
| 34379 | `timestamp` | Generic timestamp field |
| 34844 | `timestamp` | Generic timestamp field |

**Suggested Type Alias:**
```rust
type ParseOptionalUtcDateTime = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

---

### Pattern B: Result-wrapped HashMap (4 warnings)

**Pattern:** `Result<::std::collections::HashMap<String, CustomType>, String>`

**Description:** Fields that deserialize string-keyed HashMaps from JSON, with `Ok(HashMap)` for valid maps and `Err(String)` for parse errors.

**Occurrences:**

| Line | Field | Value Type |
|------|-------|------------|
| 26864 | `adapters` | `super::HoopConfigPricingAdaptersValue` |
| 26909 | `models` | `super::HoopConfigPricingAdaptersValueModelsValue` |
| 27290 | `adapters` | `super::HoopConfigStuckDetectorAdaptersValue` |
| 28964 | (field name not visible) | `super::PricingConfigAdaptersValue` |
| 29006 | `models` | `super::PricingConfigAdaptersValueModelsValue` |

**Suggested Type Alias:**
```rust
type ParseStringHashMap<T> = Result<::std::collections::HashMap<String, T>, String>;
```

---

### Pattern C: Result-wrapped Optional JSON Map (1 warning)

**Pattern:** `Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>`

**Description:** Complex nested JSON structure (tool_use field) representing optional JSON object data.

**Occurrences:**

| Line | Field | Context |
|------|-------|---------|
| 31100 | `tool_use` | Tool use data (likely Claude Code tool call) |

**Suggested Type Alias:**
```rust
type ParseOptionalJsonMap = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

---

### Pattern D: Result-wrapped serde_json::Map (1 warning)

**Pattern:** `Result<::serde_json::Map<String, ::serde_json::Value>, String>`

**Description:** Non-optional JSON Map field (args field).

**Occurrences:**

| Line | Field | Context |
|------|-------|---------|
| 22064 | `args` | Arguments field |

**Suggested Type Alias:**
```rust
type ParseJsonMap = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

---

### Pattern E: Complex Database Query Tuple (1 warning)

**Pattern:** `(String, Vec<Box<dyn rusqlite::types::ToSql>>)`

**Description:** SQL parameter tuple with trait object. This is in handwritten code (not generated).

**Occurrences:**

| File | Line | Context |
|------|------|---------|
| `hoop-mcp/src/tools.rs` | 917 | SQL query parameter construction |

**Code Context:**
```rust
let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(kind) = 
```

**Suggested Type Alias:**
```rust
type SqlParams = Vec<Box<dyn rusqlite::types::ToSql>>;
type SqlQuery = (String, SqlParams);
```

---

## Detailed File-by-File Listing

### hoop-schema (generated)

**File:** `/home/coding/target/debug/build/hoop-schema-eec83911af1a1b70/out/types.rs`

**Note:** This is **generated code** from the `hoop-schema` build script (code generation from JSON schemas). These types cannot be directly edited; fixes must be made in the code generator or schema definitions.

| Line | Field | Type | Pattern |
|------|-------|------|---------|
| 22064 | `args` | `Result<::serde_json::Map<String, ::serde_json::Value>, String>` | Pattern D |
| 22949 | `window_end` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 22950 | `window_start` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 24115 | `timestamp` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 25066 | `last_success_iso` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 26864-26867 | `adapters` | `Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValue>, String>` | Pattern B |
| 26909-26912 | `models` | `Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValueModelsValue>, String>` | Pattern B |
| 27290-27293 | `adapters` | `Result<::std::collections::HashMap<String, super::HoopConfigStuckDetectorAdaptersValue>, String>` | Pattern B |
| 28206 | `timestamp` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28465 | `closed_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28467 | `deadline` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28478 | `updated_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28692 | `added_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28787 | `created_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 28964 | (field) | `Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValue>, String>` | Pattern B |
| 29006-29009 | `models` | `Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValueModelsValue>, String>` | Pattern B |
| 29763 | `approved_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 29765 | `archived_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 29769 | `last_applied` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30409 | `timestamp` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30576 | `archived_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30579 | `closed_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30590 | `updated_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30846 | `linked_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 30970 | `created_at` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 31100 | `tool_use` | `Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>` | Pattern C |
| 31597 | `end` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 31598 | `start` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 32848 | `agent_first_used` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 32849 | `mic_first_used` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 32850 | `patterns_first_used` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 32851 | `reflection_ledger_first_used` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 34379 | `timestamp` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |
| 34844 | `timestamp` | `Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>` | Pattern A |

### hoop-mcp (handwritten)

**File:** `hoop-mcp/src/tools.rs`

| Line | Variable | Type | Pattern |
|------|----------|------|---------|
| 917 | `(sql, params)` | `(String, Vec<Box<dyn rusqlite::types::ToSql>>)` | Pattern E |

**Note:** This is **handwritten code** in the MCP server's database query construction.

---

## Recommended Next Steps

### For hoop-schema (35 warnings in generated code)

Since these types are generated from JSON schemas, the fix should be in the **code generator** (`hoop-schema/build.rs` or related codegen):

1. **Add type alias generation** to the schema code generator for common patterns:
   - `ParseOptionalUtcDateTime` for Pattern A
   - `ParseStringHashMap<T>` for Pattern B
   - `ParseOptionalJsonMap` for Pattern C
   - `ParseJsonMap` for Pattern D

2. **Generator modification approach:**
   - Detect repeated type patterns during code generation
   - Emit module-level type aliases for common patterns
   - Reference aliases instead of full types in struct fields

### For hoop-mcp (1 warning in handwritten code)

This warning is in `hoop-mcp/src/tools.rs:917`. The suggested type alias:

```rust
type SqlParams = Vec<Box<dyn rusqlite::types::ToSql>>;
type SqlQuery = (String, SqlParams);

// Usage:
let (sql, params): SqlQuery = if let Some(kind) = 
```

This can be directly applied to the handwritten code.

---

## Impact Assessment

- **hoop-schema**: 35 warnings, all in generated code. Cosmetic issue; the types work correctly but are verbose. Fix requires codegen changes.
- **hoop-mcp**: 1 warning in handwritten code. Low priority; can be cleaned up with a type alias.

**Overall Risk:** Low. These are type complexity warnings, not correctness issues. The code compiles and functions correctly; clippy is flagging verbosity that could impair readability.

---

## Generated Metadata

- **Parsed from:** `docs/type-complexity-raw.txt`
- **Total warnings:** 36
- **Pattern types identified:** 5
- **Date extracted:** 2026-08-12
