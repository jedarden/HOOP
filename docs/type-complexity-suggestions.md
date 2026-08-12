# Type Alias Suggestions for Clippy Type Complexity Warnings

**Generated:** 2026-08-12  
**Bead:** bf-7fubb  
**Source:** Analysis of 33 type complexity warnings in generated code  
**Approach:** One representative suggestion per pattern (6 total)

## Overview

All 33 type complexity warnings appear in **generated code** (`hoop-schema/build/out/types.rs`) from JSON Schema via `typify`. This document proposes **one semantic type alias per unique pattern** (6 total), not per instance.

## The 6 Representative Patterns

### 1. Event Timestamp Fields (27 instances)

**Pattern:**
```rust
Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>
```

**Example Fields:** `window_end`, `timestamp`, `closed_at`, `created_at`, `updated_at`, `deadline`, etc.

**Suggested Type Alias:** `ParsedEventTimestamp`

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
- **"Parsed"** indicates this is deserialization output, not raw input
- **"Event"** ties to HOOP's event-driven architecture
- **"Timestamp"** is accurate and domain-standard
- Captures the three-state nature (present/missing/invalid) of HOOP timestamp data
- Short enough to use in struct definitions without line breaks

**Alternative Names Considered:**
- `OptionalUtcDateTimeResult` — Too mechanical, loses domain context
- `TimestampField` — Too generic, doesn't indicate Result nature
- `EventTime` — Nice but might imply only time-of-day, not full datetime
- `HoopTimestamp` — Good alternative, emphasizes HOOP-specific usage

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

---

### 2. Pricing Adapters Configuration (1 instance)

**Pattern:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValue>, String>
```

**Example Fields:** `adapters` (in HoopConfig pricing section)

**Suggested Type Alias:** `PricingAdaptersConfig`

```rust
/// Parsed pricing adapter configuration map.
///
/// Maps adapter names to their pricing configurations.
/// The Result wrapper indicates parsing failures; the String error
/// contains schema validation messages for operator visibility.
type PricingAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, String>;
```

**Rationale:**
- **"Pricing"** indicates the config domain
- **"Adapters"** is HOOP terminology for LLM adapters
- **"Config"** indicates this is configuration data
- Specific to pricing, avoiding generics that lose context

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type PricingAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, String>;
```

---

### 3. Stuck Detector Adapters Configuration (1 instance)

**Pattern:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigStuckDetectorAdaptersValue>, String>
```

**Example Fields:** `adapters` (in HoopConfig stuck_detector section)

**Suggested Type Alias:** `StuckDetectorAdaptersConfig`

```rust
/// Parsed stuck detector adapter configuration map.
///
/// Maps adapter names to their stuck detection configurations.
/// The Result wrapper indicates parsing failures; the String error
/// contains schema validation messages for operator visibility.
type StuckDetectorAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;
```

**Rationale:**
- **"StuckDetector"** matches the config section name
- **"Adapters"** is HOOP terminology
- **"Config"** indicates this is configuration data
- Explicitly names the detector type rather than using generics

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type StuckDetectorAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;
```

---

### 4. Pricing Models Configuration (1 instance)

**Pattern:**
```rust
Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValueModelsValue>, String>
```

**Example Fields:** `models` (in PricingConfig adapters section)

**Suggested Type Alias:** `PricingModelsConfig`

```rust
/// Parsed pricing model configuration map.
///
/// Maps model names to their pricing/capacity configurations.
/// Used within pricing adapters to define per-model settings.
/// The Result wrapper indicates parsing failures.
type PricingModelsConfig = Result<::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;
```

**Rationale:**
- **"Pricing"** indicates the domain
- **"Models"** is HOOP's term for LLM models
- **"Config"** indicates this is configuration data
- Clear distinction from adapter-level configs

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type PricingModelsConfig = Result<::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;
```

---

### 5. Tool Arguments JSON (1 instance)

**Pattern:**
```rust
Result<::serde_json::Map<String, ::serde_json::Value>, String>
```

**Example Fields:** `args` (in tool invocation messages)

**Suggested Type Alias:** `ToolArgsMap`

```rust
/// Parsed tool arguments from a Claude tool use call.
///
/// Represents the structured arguments passed to a Claude Code tool.
/// The Result wrapper captures JSON parsing errors; the String error
/// contains the invalid JSON for debugging and operator feedback.
///
/// This is the non-optional variant (tool calls always have args).
type ToolArgsMap = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

**Rationale:**
- **"Tool"** ties to Claude Code tool calls
- **"Args"** is the exact field name and purpose
- **"Map"** indicates the JSON Map structure
- Clear and concise for this specific use case

**Alternative Names Considered:**
- `ParsedToolArgs` — Good, slightly more verbose
- `ToolCallArgs` — Might imply call metadata too
- `JsonMapResult` — Too generic, loses tool context

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type ToolArgsMap = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

---

### 6. Optional Tool Use JSON (1 instance)

**Pattern:**
```rust
Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>
```

**Example Fields:** `tool_use` (in message content)

**Suggested Type Alias:** `OptionalToolUseMap`

```rust
/// Parsed optional tool use data from a message.
///
/// Represents tool_use content that may be present, absent, or invalid.
/// The three-state Result<Option<>> structure matches Claude message semantics:
/// - Ok(Some(Map)): Valid tool use content
/// - Ok(None): No tool use in this message (e.g., text-only)
/// - Err(String): Invalid tool_use JSON
///
/// Distinct from `ToolArgsMap` which is always present in tool calls.
type OptionalToolUseMap = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

**Rationale:**
- **"Optional"** clearly indicates the Option wrapper
- **"ToolUse"** matches Claude message terminology
- **"Map"** indicates the JSON Map structure
- Explicit three-state semantics (present/absent/invalid)

**Alternative Names Considered:**
- `ParsedToolUse` — Good, but loses the "Optional" hint
- `ToolUseResult` — Doesn't convey the Option wrapper
- `MessageToolUse` — Accurate but more verbose

**Where to Define:**
```rust
// hoop-schema/src/lib.rs or hoop-schema/src/aliases.rs
pub type OptionalToolUseMap = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

---

## Implementation Strategy

### Recommended Approach: Separate Type Alias Module

Create `hoop-schema/src/aliases.rs` and re-export from `lib.rs`:

```rust
// hoop-schema/src/aliases.rs
//! Type aliases for complex generated types.

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use std::collections::HashMap;

// Event timestamps (27 instances)
pub type ParsedEventTimestamp = Result<Option<DateTime<Utc>>, String>;

// Configuration maps (3 instances)
pub type PricingAdaptersConfig = Result<HashMap<String, HoopConfigPricingAdaptersValue>, String>;
pub type StuckDetectorAdaptersConfig = Result<HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;
pub type PricingModelsConfig = Result<HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;

// Tool JSON maps (2 instances)
pub type ToolArgsMap = Result<Map<String, Value>, String>;
pub type OptionalToolUseMap = Result<Option<Map<String, Value>>, String>;

// hoop-schema/src/lib.rs
pub mod aliases;
pub use aliases::{
    ParsedEventTimestamp,
    PricingAdaptersConfig,
    StuckDetectorAdaptersConfig,
    PricingModelsConfig,
    ToolArgsMap,
    OptionalToolUseMap,
};
```

### Alternative: Add to Build Script

Modify `hoop-schema/build.rs` to append aliases after `typify` generation:

```rust
// After typify generation, append aliases
let aliases = r#"
// Type aliases for complex generated types (see docs/type-complexity-suggestions.md)
pub type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
pub type PricingAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, String>;
pub type StuckDetectorAdaptersConfig = Result<::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;
pub type PricingModelsConfig = Result<::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;
pub type ToolArgsMap = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
pub type OptionalToolUseMap = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
"#;

// Append to generated types.rs
std::fs::write("build/out/types.rs", format!("{}\n{}", generated_code, aliases))?;
```

**Recommendation:** Use the separate module approach. It's cleaner, more discoverable, and doesn't fight the code generator.

---

## Naming Convention Summary

All aliases follow Rust naming conventions:

1. **PascalCase** for type names (e.g., `ParsedEventTimestamp`)
2. **Descriptive and semantic** — domain-relevant, not mechanical (e.g., `ToolArgsMap` not `JsonMapResult1`)
3. **Consistent terminology** — "Config" for configuration, "Parsed" for deserialization output
4. **Explicit about wrappers** — "Optional" in name when `Option<>` is present
5. **Clear distinction** — `ToolArgsMap` vs `OptionalToolUseMap` makes the difference obvious

---

## Migration Example

**Before (with type complexity warning):**
```rust
pub struct Message {
    pub timestamp: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>,
    pub tool_use: Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>,
}
```

**After (with type aliases):**
```rust
pub struct Message {
    pub timestamp: ParsedEventTimestamp,
    pub tool_use: OptionalToolUseMap,
}
```

**Benefits:**
- **68% reduction** in type signature verbosity (68 chars → 22 chars for timestamps)
- **Self-documenting** field types
- **Clear domain semantics** — intent is obvious at a glance
- **Easier code review** — no mental parsing of complex types

---

## Verification

After implementing these aliases:

1. **Regenerate types:** Re-run the schema generation build
2. **Verify compilation:** `cargo check --workspace`
3. **Run clippy:** `cargo clippy --workspace -- -W type_complexity` — should show 0 warnings
4. **Run tests:** `cargo test --workspace` — ensure all deserialization tests pass
5. **Document:** Add rustdoc comments for each alias

---

## Related Documentation

- **Raw warnings:** `docs/type-complexity-raw.txt` — Original clippy output
- **Parsed analysis:** `docs/type-complexity-parsed.md` — Pattern categorization
- **Detailed proposals:** `docs/type-complexity-aliases.md` — Alternative alias names with full rationale
- **Schema definitions:** `hoop-schema/schemas/*.json` — Source JSON schemas
- **Build configuration:** `hoop-schema/build.rs` — Code generation script

---

## Bead Context

**Task:** bf-7fubb — Generate type alias suggestions for each warning  
**Dependency:** Child 2 — Parse and categorize type complexity warnings  
**Acceptance:** Each of the 6 warnings has a suggested type alias name with rationale  
**Blocks:** Child 4 — Write type complexity catalog markdown  
**Status:** Complete — 6 representative patterns documented with semantic aliases
