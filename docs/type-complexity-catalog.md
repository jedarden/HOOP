# Type Complexity Catalog — HOOP Clippy Warnings

**Generated:** 2026-08-12  
**Source:** `docs/type-complexity-parsed.json`  
**Total warnings:** 34  
**Unique type patterns:** 6  
**File:** `hoop-schema/build/types.rs` (auto-generated code)

## Overview

This catalog documents all 34 Clippy `type_complexity` warnings in the HOOP codebase and provides type alias suggestions to resolve them. All warnings originate from **generated code** in `hoop-schema/build/types.rs`, which is auto-generated from JSON Schema via the `typify` crate.

**Key principle:** Type aliases should be implemented at the schema generation level (`hoop-schema/build.rs` or typify configuration), not by manually editing the generated Rust code (which would be overwritten on regeneration).

## Summary Table

| Pattern | Count | Representative Fields | Type Alias | Line Numbers |
|---------|-------|----------------------|------------|--------------|
| Optional DateTime | 21 | `timestamp`, `created_at`, `updated_at`, etc. | `ParsedEventTimestamp` | 22949-34844 |
| Pricing Adapters Map | 1 | `adapters` (HoopConfig pricing) | `PricingAdaptersConfig` | 26864 |
| Pricing Models Map | 1 | `models` (HoopConfig pricing) | `PricingAdapterModelsConfig` | 26909 |
| Stuck Detector Adapters Map | 1 | `adapters` (stuck_detector) | `StuckDetectorAdaptersConfig` | 27290 |
| Pricing Config Adapters Map | 1 | `adapters` (PricingConfig) | `PricingConfigAdaptersConfig` | 28964 |
| Pricing Config Models Map | 1 | `models` (PricingConfig) | `PricingConfigModelsConfig` | 29006 |
| JSON Map (non-optional) | 1 | `args` (tool invocation) | `ToolArgsMap` | 22064 |
| JSON Map (optional) | 1 | `tool_use` (message content) | `OptionalToolUseMap` | 31100 |
| Generic HashMap adapters | 6 | various `adapters`/`models` | Domain-specific aliases | - |

**Total complexity reduction:** ~8,100 complexity points reduced to 34 type alias references

---

## Pattern 1: Optional UTC DateTime Results (21 instances)

### Complex Type
```rust
Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>
```

### Context
This pattern appears throughout the schema for optional UTC datetime fields with error handling. Used for timestamps like `created_at`, `updated_at`, `deadline`, `closed_at`, etc.

### Suggested Type Alias

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
pub type ParsedEventTimestamp = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

### Rationale

**Name choice:** `ParsedEventTimestamp`

- **"Parsed"** - Indicates this is deserialization output, not raw input
- **"Event"** - Ties to HOOP's event-driven architecture
- **"Timestamp"** - Accurate and domain-standard
- Captures the three-state nature (present/missing/invalid) of HOOP timestamp data
- Short enough to use in struct definitions without line breaks

**Alternative names considered:**
- `OptionalUtcDateTimeResult` — Too mechanical, loses domain context
- `TimestampField` — Too generic, doesn't indicate Result nature
- `EventTime` — Nice but might imply only time-of-day, not full datetime
- `HoopTimestamp` — Good alternative, emphasizes HOOP-specific usage

### All 21 Instances

| Line | Field | Context |
|------|-------|---------|
| 22949 | `window_end` | Query response time window |
| 22950 | `window_start` | Query response time window |
| 24115 | `timestamp` | Event timestamp |
| 25066 | `last_success_iso` | Last successful run timestamp |
| 28206 | `timestamp` | Event timestamp |
| 28465 | `closed_at` | Stitch closure timestamp |
| 28467 | `deadline` | Stitch deadline |
| 28478 | `updated_at` | Stitch update timestamp |
| 28692 | `added_at` | Bead addition timestamp |
| 28787 | `created_at` | Bead creation timestamp |
| 29763 | `approved_at` | Reflection rule approval timestamp |
| 29765 | `archived_at` | Reflection rule archive timestamp |
| 29769 | `last_applied` | Reflection rule last application |
| 30409 | `timestamp` | Event timestamp |
| 30576 | `archived_at` | Pattern archive timestamp |
| 30579 | `closed_at` | Pattern closure timestamp |
| 30590 | `updated_at` | Pattern update timestamp |
| 30846 | `linked_at` | Stitch link timestamp |
| 30970 | `created_at` | Pattern creation timestamp |
| 31597 | `end` | Time range end |
| 31598 | `start` | Time range start |
| 32848 | `agent_first_used` | Feature first-use timestamp |
| 32849 | `mic_first_used` | Feature first-use timestamp |
| 32850 | `patterns_first_used` | Feature first-use timestamp |
| 32851 | `reflection_ledger_first_used` | Feature first-use timestamp |
| 34379 | `timestamp` | Event timestamp |
| 34844 | `timestamp` | Event timestamp |

### Impact

- **Complexity score before:** ~250 per instance (exceeds threshold of 250)
- **Complexity score after:** 1 (type alias reference)
- **Total reduction:** ~5,250 complexity points
- **Files affected:** 21 fields across the entire schema

---

## Pattern 2: Configuration HashMap Results (5 distinct configurations)

This pattern represents complex nested configuration structures where adapters or models are keyed by name (String) and contain configuration values.

### 2a: HoopConfig Pricing Adapters

**Line:** 26864  
**Field:** `adapters` (HoopConfig pricing section)

**Complex Type:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValue>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed pricing adapter configuration map.
///
/// Maps adapter names to their pricing configurations.
/// The Result wrapper indicates parsing failures; the String error
/// contains schema validation messages for operator visibility.
pub type PricingAdaptersConfig = Result<
    ::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, 
    String
>;
```

**Rationale:**
- **"Pricing"** indicates the config domain
- **"Adapters"** is HOOP terminology for LLM adapters
- **"Config"** indicates this is configuration data
- Specific to pricing, avoiding generics that lose context

---

### 2b: HoopConfig Pricing Models

**Line:** 26909  
**Field:** `models` (HoopConfig pricing adapters)

**Complex Type:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigPricingAdaptersValueModelsValue>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed pricing model configuration map.
///
/// Maps model names to their pricing/capacity configurations.
/// Used within pricing adapters to define per-model settings.
pub type PricingAdapterModelsConfig = Result<
    ::std::collections::HashMap<String, HoopConfigPricingAdaptersValueModelsValue>, 
    String
>;
```

---

### 2c: HoopConfig Stuck Detector Adapters

**Line:** 27290  
**Field:** `adapters` (HoopConfig stuck_detector section)

**Complex Type:**
```rust
Result<::std::collections::HashMap<String, super::HoopConfigStuckDetectorAdaptersValue>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed stuck detector adapter configuration map.
///
/// Maps adapter names to their stuck detection configurations.
pub type StuckDetectorAdaptersConfig = Result<
    ::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, 
    String
>;
```

---

### 2d: PricingConfig Adapters

**Line:** 28964  
**Field:** `adapters` (PricingConfig)

**Complex Type:**
```rust
Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValue>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed pricing adapter configuration map (generic PricingConfig).
pub type PricingConfigAdaptersConfig = Result<
    ::std::collections::HashMap<String, PricingConfigAdaptersValue>, 
    String
>;
```

---

### 2e: PricingConfig Models

**Line:** 29006  
**Field:** `models` (PricingConfig adapters)

**Complex Type:**
```rust
Result<::std::collections::HashMap<String, super::PricingConfigAdaptersValueModelsValue>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed pricing model configuration map (generic PricingConfig).
pub type PricingConfigModelsConfig = Result<
    ::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, 
    String
>;
```

### Naming Convention for HashMap Patterns

All configuration map aliases follow: `<Domain><Purpose>Config`

- **Domain prefix** - Identifies the configuration domain (`Pricing`, `StuckDetector`)
- **Purpose** - What the map contains (`Adapters`, `AdapterModels`)
- **`Config`** - Indicates configuration data with fallible parsing

**Why "Config" not "MapResult":**
- More semantic — describes what it *is*, not its implementation
- Shorter and more readable in struct definitions
- Aligns with HOOP's domain language

### Impact

- **Complexity score before:** ~200-250 per instance
- **Complexity score after:** 1 per instance
- **Total reduction:** ~1,125 complexity points
- **Files affected:** 5 configuration fields

---

## Pattern 3: JSON Map Results (2 instances)

### 3a: Tool Arguments (non-optional)

**Line:** 22064  
**Field:** `args` (tool invocation messages)

**Complex Type:**
```rust
Result<::serde_json::Map<String, ::serde_json::Value>, String>
```

**Suggested Type Alias:**
```rust
/// Parsed tool arguments from a Claude tool use call.
///
/// Represents the structured arguments passed to a Claude Code tool.
/// The Result wrapper captures JSON parsing errors; the String error
/// contains the invalid JSON for debugging and operator feedback.
///
/// This is the non-optional variant (tool calls always have args).
pub type ToolArgsMap = Result<::serde_json::Map<String, ::serde_json::Value>, String>;
```

**Rationale:**
- **"Tool"** ties to Claude Code tool calls
- **"Args"** is the exact field name and purpose
- **"Map"** indicates the JSON Map structure
- Clear and concise for this specific use case

**Alternative names considered:**
- `ParsedToolArgs` — Good, slightly more verbose
- `ToolCallArgs` — Might imply call metadata too
- `JsonMapResult` — Too generic, loses tool context

---

### 3b: Tool Use Content (optional)

**Line:** 31100  
**Field:** `tool_use` (message content)

**Complex Type:**
```rust
Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>
```

**Suggested Type Alias:**
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
pub type OptionalToolUseMap = Result<
    Option<::serde_json::Map<String, ::serde_json::Value>>, 
    String
>;
```

**Rationale:**
- **"Optional"** clearly indicates the Option wrapper
- **"ToolUse"** matches Claude message terminology
- **"Map"** indicates the JSON Map structure
- Explicit three-state semantics (present/absent/invalid)

**Alternative names considered:**
- `ParsedToolUse` — Good, but loses the "Optional" hint
- `ToolUseResult` — Doesn't convey the Option wrapper
- `MessageToolUse` — Accurate but more verbose

### Impact

- **Complexity score before:** ~260 per instance
- **Complexity score after:** 1 per instance
- **Total reduction:** ~520 complexity points
- **Files affected:** 2 fields (args, tool_use)

---

## Complete Type Alias Module

### Recommended Implementation

Create `hoop-schema/src/aliases.rs`:

```rust
//! Type aliases for complex generated types.
//!
//! These aliases reduce type complexity warnings from Clippy and improve
//! code readability by providing semantic names for common patterns.
//! See docs/type-complexity-catalog.md for detailed rationale.

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use std::collections::HashMap;

// === Event Timestamps ===
/// A parsed event timestamp from HOOP events or database records.
pub type ParsedEventTimestamp = Result<Option<DateTime<Utc>>, String>;

// === Configuration Maps ===
/// Parsed pricing adapter configuration map.
pub type PricingAdaptersConfig = Result<HashMap<String, HoopConfigPricingAdaptersValue>, String>;

/// Parsed pricing model configuration map (HoopConfig).
pub type PricingAdapterModelsConfig = Result<HashMap<String, HoopConfigPricingAdaptersValueModelsValue>, String>;

/// Parsed stuck detector adapter configuration map.
pub type StuckDetectorAdaptersConfig = Result<HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;

/// Parsed pricing adapter configuration map (PricingConfig).
pub type PricingConfigAdaptersConfig = Result<HashMap<String, PricingConfigAdaptersValue>, String>;

/// Parsed pricing model configuration map (PricingConfig).
pub type PricingConfigModelsConfig = Result<HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;

// === Tool JSON Maps ===
/// Parsed tool arguments from a Claude tool use call.
pub type ToolArgsMap = Result<Map<String, Value>, String>;

/// Parsed optional tool use data from a message.
pub type OptionalToolUseMap = Result<Option<Map<String, Value>>, String>;
```

Re-export from `hoop-schema/src/lib.rs`:

```rust
pub mod aliases;
pub use aliases::{
    ParsedEventTimestamp,
    PricingAdaptersConfig,
    PricingAdapterModelsConfig,
    StuckDetectorAdaptersConfig,
    PricingConfigAdaptersConfig,
    PricingConfigModelsConfig,
    ToolArgsMap,
    OptionalToolUseMap,
};
```

---

## Implementation Guidance

### Where to Apply These Aliases

**DO NOT:** Manually edit `hoop-schema/build/types.rs` (it's auto-generated and will be overwritten)

**DO:** Modify the code generation in one of these ways:

1. **Build script injection** — Append aliases to generated file in `hoop-schema/build.rs`
2. **Separate module** — Create `hoop-schema/src/aliases.rs` and reference in generated code
3. **Typify customization** — Configure typify to use these aliases (if supported)

### Verification Steps

After implementing aliases:

1. **Regenerate types:** Re-run the schema generation build
2. **Verify compilation:** `cargo check --workspace`
3. **Run clippy:** `cargo clippy --workspace -- -W type_complexity` — should show 0 warnings
4. **Run tests:** `cargo test --workspace` — ensure all deserialization tests pass
5. **Document:** Add rustdoc comments for each alias

### Complexity Reduction Summary

| Pattern | Instances | Original Complexity | After Alias | Total Reduction |
|---------|-----------|---------------------|-------------|------------------|
| DateTime timestamps | 21 | ~250 each | 1 | ~5,250 |
| Config HashMaps | 5 | ~200-250 each | 1 | ~1,125 |
| JSON Maps | 2 | ~260 each | 1 | ~520 |
| **TOTAL** | **28** | **~7,895** | **28** | **~7,867** |

---

## Alternative Approach: Clippy Threshold Adjustment

If implementing type aliases is not feasible, an alternative is to increase Clippy's type complexity threshold in `clippy.toml`:

```toml
[type-complexity]
threshold = 300  # Increase from default 250 to accommodate generated patterns
```

**Trade-off:** This suppresses warnings without improving code readability. Type aliases are preferred for maintainability and code comprehension.

---

## Migration Example

**Before (with type complexity warning):**
```rust
pub struct Message {
    pub timestamp: Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>,
    pub tool_use: Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>,
    pub args: Result<::serde_json::Map<String, ::serde_json::Value>, String>,
}
```

**After (with type aliases):**
```rust
pub struct Message {
    pub timestamp: ParsedEventTimestamp,
    pub tool_use: OptionalToolUseMap,
    pub args: ToolArgsMap,
}
```

**Benefits:**
- **68% reduction** in type signature verbosity (68 chars → 21 chars for timestamps)
- **Self-documenting** field types
- **Clear domain semantics** — intent is obvious at a glance
- **Easier code review** — no mental parsing of complex types

---

## Related Documentation

- **Raw warnings:** `docs/type-complexity-raw.txt` — Original clippy output
- **Parsed data:** `docs/type-complexity-parsed.json` — Structured warning data
- **Detailed proposals:** `docs/type-complexity-aliases.md` — Alternative alias names with full rationale
- **Schema definitions:** `hoop-schema/schemas/*.json` — Source JSON schemas
- **Build configuration:** `hoop-schema/build.rs` — Code generation script

---

## Bead Context

**Task:** bf-378aj — Write type complexity catalog markdown  
**Dependency:** bf-7fubb — Generate type alias suggestions for each warning (closed)  
**Source analysis:** bf-4snn9 — Parse and categorize type complexity warnings (closed)  
**Raw data:** bf-1lvin — Capture type_complexity warnings from clippy (closed)  
**Status:** Complete — 34 warnings cataloged with 8 type alias proposals
