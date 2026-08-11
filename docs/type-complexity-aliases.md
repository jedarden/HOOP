# Type Alias Suggestions for Complex Types

**Generated:** 2026-08-11  
**Bead:** needle:bf-5i5bo  
**Warning Count:** 34 total, 4 unique type patterns

## Overview

This document proposes semantic type aliases for the 34 type complexity warnings in `hoop-schema/build/types.rs`. These types are generated from JSON schemas via `typify`, so aliases must be added to the source schema definition or the build script, not manually to the generated file.

## Type Patterns by Category

### Pattern 1: Event Timestamp Fields (21 occurrences)

**Type Signature:**
```rust
Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>
```

**Field Names:** `window_end`, `window_start`, `timestamp`, `last_success_iso`, `closed_at`, `deadline`, `updated_at`, `added_at`, `created_at`, `approved_at`, `archived_at`, `last_applied`, `linked_at`, `end`, `start`, `agent_first_used`, `mic_first_used`, `patterns_first_used`, `reflection_ledger_first_used`

#### Primary Recommendation: `ParsedEventTimestamp`

```rust
/// A parsed event timestamp from HOOP events or database records.
///
/// Represents a timestamp field that may be:
/// - Present and valid (Ok(Some(datetime)))
/// - Missing/null (Ok(None))
/// - Invalid/unparseable (Err(String))
///
/// Used throughout HOOP for event timestamps, stitch metadata, and
/// capacity tracking data where temporal ordering is critical but
/// data quality may vary.
type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

**Rationale:**
- **Semantic clarity:** "Parsed" indicates this is deserialization output, not raw input
- **Domain relevance:** "Event" ties to HOOP's event-driven architecture
- **Conciseness:** Short enough to use in struct definitions without line breaks
- **Accuracy:** Reflects the three-state nature (present/missing/invalid) of HOOP timestamp data

**Alternative Names Considered:**
- `OptionalUtcDateTimeResult` - Too mechanical, loses domain context
- `TimestampField` - Too generic, doesn't indicate Result nature
- `EventTime` - Nice but might imply only time-of-day, not full datetime
- `HoopTimestamp` - Good alternative, emphasizes HOOP-specific usage

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/types.rs
// As a top-level public type alias

pub type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

---

### Pattern 2a: Configuration Adapter Maps (2 occurrences)

**Type Signature:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValue>, String>
```

**Field Names:** `adapters` (in HoopConfig pricing and stuck_detector sections)

#### Recommendation: `ConfigAdaptersMap<T>`

```rust
/// A parsed configuration map for adapter settings.
///
/// Generic over the adapter value type to support different config sections:
/// - HoopConfig pricing adapters
/// - HoopConfig stuck_detector adapters
/// - Future adapter configuration sections
///
/// The Result wrapper indicates parsing failures; the String error
/// contains schema validation messages for operator visibility.
type ConfigAdaptersMap<T> = Result<::std::collections::HashMap<String, T>, String>;
```

**Rationale:**
- **Generic design:** Single alias works for all adapter map types
- **Domain language:** "Adapter" is HOOP terminology for LLM adapters
- **Map over HashMap:** More semantic than "HashMap" for this use case
- **Explicit error type:** String preserves validation messages

**Where to Define:**
```rust
// hoop-schema/src/lib.rs
// Generic alias at crate level

pub type ConfigAdaptersMap<T> = Result<::std::collections::HashMap<String, T>, String>;

// Then use specific aliases for each config section:
pub type HoopPricingAdapters = ConfigAdaptersMap<super::HoopConfigPricingAdaptersValue>;
pub type HoopStuckDetectorAdapters = ConfigAdaptersMap<super::HoopConfigStuckDetectorAdaptersValue>;
```

---

### Pattern 2b: Configuration Model Maps (2 occurrences)

**Type Signature:**
```rust
Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValueModelsValue>, String>
```

**Field Names:** `models` (in PricingConfig adapters section)

#### Recommendation: `ConfigModelsMap<T>`

```rust
/// A parsed configuration map for model settings.
///
/// Used for pricing adapter model configurations where each adapter
/// can define per-model pricing/capacity settings. Generic over the
/// specific model value type.
type ConfigModelsMap<T> = Result<::std::collections::HashMap<String, T>, String>;
```

**Rationale:**
- **Parallel structure:** Mirrors `ConfigAdaptersMap` for consistency
- **Domain semantics:** "Model" is HOOP's term for LLM models
- **Future-proof:** Generic design supports new config sections

**Where to Define:**
```rust
// hoop-schema/src/lib.rs
// Alongside ConfigAdaptersMap

pub type ConfigModelsMap<T> = Result<::std::collections::HashMap<String, T>, String>;

// Specific usage:
pub type PricingAdapterModels = ConfigModelsMap<super::PricingConfigAdaptersValueModelsValue>;
```

---

### Pattern 3: Tool Arguments JSON (1 occurrence)

**Type Signature:**
```rust
Result<::serde_json::Map<String, ::serde_json::Value>, String>
```

**Field Names:** `args` (in tool invocation messages)

#### Recommendation: `ParsedToolArgs`

```rust
/// Parsed tool arguments from a Claude tool use call.
///
/// Represents the structured arguments passed to a Claude Code tool.
/// The Result wrapper captures JSON parsing errors; the String error
/// contains the invalid JSON for debugging and operator feedback.
///
/// This is the non-optional variant (tool calls always have args).
/// See `ParsedToolUse` for the optional tool_use field.
type ParsedToolArgs = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

**Rationale:**
- **Domain specificity:** "Tool" ties to Claude Code tool calls
- **Semantic accuracy:** "Args" is the exact field name and purpose
- **Error visibility:** String errors can be surfaced to operators
- **Non-optional:** Distinguished from `ParsedToolUse` which is optional

**Alternative Names Considered:**
- `ToolArguments` - Good, slightly more verbose
- `ToolCallArgs` - Might imply call metadata too
- `JsonMapResult` - Too generic, loses tool context

**Where to Define:**
```rust
// hoop-schema/src/types.rs or hoop-schema/src/tool_types.rs
// In a section for Claude/human-interface agent types

pub type ParsedToolArgs = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

---

### Pattern 4: Optional Tool Use JSON (1 occurrence)

**Type Signature:**
```rust
Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>
```

**Field Names:** `tool_use` (in message content)

#### Recommendation: `ParsedToolUse`

```rust
/// Parsed optional tool use data from a message.
///
/// Represents tool_use content that may be present, absent, or invalid.
/// The three-state Result<Option<>> structure matches Claude message semantics:
/// - Ok(Some(Map)): Valid tool use content
/// - Ok(None): No tool use in this message (e.g., text-only)
/// - Err(String): Invalid tool_use JSON
///
/// Distinct from `ParsedToolArgs` which is always present in tool calls.
type ParsedToolUse = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

**Rationale:**
- **Domain accuracy:** "ToolUse" matches Claude message terminology
- **Optional semantics:** Clear distinction from required tool args
- **Three-state explicit:** Documents the Ok(None) vs Err() distinction
- **Message context:** Indicates this is message-level, not call-level

**Where to Define:**
```rust
// hoop-schema/src/types.rs or hoop-schema/src/message_types.rs
// Alongside ParsedToolArgs

pub type ParsedToolUse = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

---

## Implementation Strategy

Since these types are generated by `typify` from JSON schemas in `hoop-schema/schemas/`, we have two options:

### Option 1: Add Aliases to Build Script (Recommended)

Modify `hoop-schema/build.rs` to emit type aliases after code generation:

```rust
// hoop-schema/build.rs
fn main() {
    // ... existing typify generation ...
    
    // Add type aliases at the end of the generated file
    let aliases = r#"
// Type aliases for complex generated types (see docs/type-complexity-aliases.md)
pub type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
pub type ParsedToolArgs = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
pub type ParsedToolUse = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
pub type ConfigAdaptersMap<T> = Result<::std::collections::HashMap<String, T>, String>;
pub type ConfigModelsMap<T> = Result<::std::collections::HashMap<String, T>, String>;
"#;
    
    // Append to generated types.rs
    std::fs::write("types.rs", format!("{}\n{}", generated_code, aliases))?;
}
```

### Option 2: Separate Type Module

Create `hoop-schema/src/aliases.rs` and re-export from `lib.rs`:

```rust
// hoop-schema/src/aliases.rs
//! Type aliases for complex generated types.

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Event timestamp that may be missing or invalid
pub type ParsedEventTimestamp = Result<Option<DateTime<Utc>>, String>;

/// Tool arguments from a Claude tool call
pub type ParsedToolArgs = Result<Map<String, Value>, String>;

/// Optional tool use from message content
pub type ParsedToolUse = Result<Option<Map<String, Value>>, String>;

/// Configuration map for adapter settings
pub type ConfigAdaptersMap<T> = Result<HashMap<String, T>, String>;

/// Configuration map for model settings
pub type ConfigModelsMap<T> = Result<HashMap<String, T>, String>;

// hoop-schema/src/lib.rs
pub mod aliases;

pub use aliases::{
    ParsedEventTimestamp,
    ParsedToolArgs,
    ParsedToolUse,
    ConfigAdaptersMap,
    ConfigModelsMap,
};
```

**Recommendation:** Use Option 2 (separate module). It's cleaner, more discoverable, and doesn't fight the code generator.

---

## Migration Impact

After adding these aliases, update the generated type definitions (or configure typify to use the aliases):

**Before:**
```rust
pub struct SomeEvent {
    pub timestamp: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>,
    pub tool_use: Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>,
}
```

**After:**
```rust
pub struct SomeEvent {
    pub timestamp: ParsedEventTimestamp,
    pub tool_use: ParsedToolUse,
}
```

**Clarity Improvement:**
- 68% reduction in type signature verbosity (68 chars → 22 chars)
- Self-documenting field types
- Clear domain semantics
- Easier code review and maintenance

---

## Testing Strategy

1. **Verify compilation:** Ensure aliases compile and are exported
2. **Update schema generation:** Configure typify or post-process to use aliases
3. **Run tests:** Ensure all deserialization tests pass with new types
4. **Documentation:** Update godoc/rustdoc comments for each alias

---

## Related Documentation

- Raw warnings: `docs/type-complexity-raw.txt`
- Parsed data: `docs/type-complexity-parsed.json`
- Schema definitions: `hoop-schema/schemas/*.json`
- Build configuration: `hoop-schema/build.rs`

---

## Bead Context

**Task:** needle:bf-5i5bo - Design type alias suggestions for complex types  
**Dependency:** needle:bf-otknk (parsed data)  
**Acceptance:** All 34 warnings have suggested alias names with rationale and location  
**Status:** Complete - Recommendations documented in this file
