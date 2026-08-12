# Type Complexity Catalog — Suggested Aliases

**Generated:** 2026-08-12  
**Source:** `docs/type-complexity-parsed.md`  
**Total warnings:** 33  
**Patterns identified:** 3

## Overview

This catalog provides type alias suggestions for all Clippy type complexity warnings in the HOOP codebase. All warnings originate from **generated code** in `hoop-schema/build/out/types.rs`, which is auto-generated from JSON Schema via `typify`.

**Key principle:** These aliases should be implemented at the schema generation level (`hoop-schema/build.rs` or typify configuration), not by manually editing the generated Rust code.

---

## Pattern 1: Optional DateTime Results (27 instances)

### Complex Type
```rust
Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>
```

### Context
This pattern appears throughout the schema for optional UTC datetime fields with error handling. Used for timestamps like `created_at`, `updated_at`, `deadline`, etc.

### Suggested Type Alias

```rust
/// A Result wrapping an optional UTC DateTime with String error type.
/// Used for timestamp fields that may be null or parsing may fail.
type OptionalUtcDateTimeResult = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
```

### Rationale

**Name choice:** `OptionalUtcDateTimeResult`

- **`Optional`** - Clearly signals the inner `Option<>`, indicating the field may be null/absent
- **`UtcDateTime`** - Describes the payload: a timezone-aware UTC timestamp (more precise than just "DateTime")
- **`Result`** - Indicates fallibility (parsing/validation can fail with a String error)
- **Ordering** - Flows from outermost to innermost: Optional → UtcDateTime → Result

**Why not alternatives considered:**

- `OptionalDateTimeResult` - Less precise; doesn't indicate UTC timezone
- `TimestampResult` - Too vague; doesn't convey Option wrapper
- `DateTimeOrError` - Non-idiomatic; Rust convention puts the success type first
- `ParsedTimestamp` - Doesn't convey the Result/Option structure clearly

**Semantic meaning:** "A nullable UTC timestamp that may fail to parse, returning a String error message."

### Usage Example

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EventMetadata {
    created_at: OptionalUtcDateTimeResult,
    updated_at: OptionalUtcDateTimeResult,
    deadline: OptionalUtcDateTimeResult,
}
```

### Impact

**Reduces complexity score from:** ~250 (exceeds threshold)  
**Reduces to:** 1 (type alias reference)  
**Affects:** 27 fields across the schema

---

## Pattern 2: Adapter Configuration Maps (5 instances)

### Complex Type
```rust
Result<::std::collections::HashMap<String, SomeAdapterType>, String>
```

### Context
These represent complex nested configuration structures where adapters are keyed by name (String) and contain configuration values. This pattern appears in pricing and stuck detector configurations.

### Suggested Type Aliases

```rust
/// A Result wrapping a HashMap of pricing adapter configurations.
/// Keys are adapter names, values contain pricing-specific adapter settings.
type PricingAdaptersMapResult = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, String>;

/// A Result wrapping a HashMap of pricing model configurations within adapters.
/// Keys are model names, values contain model-specific pricing settings.
type PricingAdapterModelsMapResult = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValueModelsValue>, String>;

/// A Result wrapping a HashMap of stuck detector adapter configurations.
/// Keys are adapter names, values contain detector-specific settings.
type StuckDetectorAdaptersMapResult = Result<::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;

/// A Result wrapping a HashMap of generic pricing adapter configurations.
/// Used in PricingConfig for adapter overrides.
type GenericPricingAdaptersMapResult = Result<::std::collections::HashMap<String, PricingConfigAdaptersValue>, String>;

/// A Result wrapping a HashMap of generic pricing model configurations.
/// Used in PricingConfig for model-specific overrides.
type GenericPricingAdapterModelsMapResult = Result<::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;
```

### Rationale

**Name choice pattern:** `<Domain><Purpose>MapResult`

- **Domain prefix** - Identifies the configuration domain (`Pricing`, `StuckDetector`, `GenericPricing`)
- **Purpose** - What the map contains (`Adapters`, `AdapterModels`, `StuckDetector`)
- **`Map`** - Indicates HashMap structure (key-value pairs)
- **`Result`** - Signals fallibility with String error type

**Why `Map` not `HashMap`:** More abstract; allows changing the implementation from HashMap to BTreeMap without updating the alias name.

**Why `Result` suffix:** Clearly indicates this is a fallible operation, not just a container.

**Semantic meaning:** "A configuration map of domain-specific adapters/models that may fail to load/parse, returning a String error."

### Usage Example

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct HoopConfigPricing {
    adapters: PricingAdaptersMapResult,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct HoopConfigStuckDetector {
    adapters: StuckDetectorAdaptersMapResult,
}
```

### Impact

**Reduces complexity score from:** ~200-250 per instance  
**Reduces to:** 1 (type alias reference)  
**Affects:** 5 fields across pricing and stuck detector configurations

---

## Pattern 3: Optional JSON Map Results (1 instance)

### Complex Type
```rust
Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>
```

### Context

This pattern appears in the `tool_use` field, representing structured JSON data that may be null and may fail to parse. It's distinct from Pattern 1 because it uses `serde_json::Map` instead of `chrono::DateTime`.

### Suggested Type Alias

```rust
/// A Result wrapping an optional JSON object (Map of String to Value) with String error type.
/// Used for JSON payloads that may be null or parsing may fail.
type OptionalJsonObjectResult = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

### Rationale

**Name choice:** `OptionalJsonObjectResult`

- **`Optional`** - Signals the inner `Option<>`; field may be null
- **`JsonObject`** - Describes the payload: a JSON object (Map<String, Value>)
  - Not `JsonMap` - "Object" is the semantic JSON type name; "Map" is the implementation detail
  - Not `JsonData` - Too vague; doesn't convey it's an object (could be an array)
- **`Result`** - Indicates fallibility with String error type

**Why "JsonObject" not "JsonMap":**
- In JSON terminology, "object" is the standard name for `{ "key": "value" }` structures
- `serde_json::Map` is the Rust implementation; users think in JSON terms
- Aligns with semantic naming: we're describing a JSON object, not a HashMap

**Why not alternatives:**
- `OptionalJsonMapResult` - "Map" is an implementation detail, not the domain concept
- `OptionalJsonResult` - Too vague; could be a JSON array or string
- `JsonToolUseResult` - Too specific to one field; not reusable

**Semantic meaning:** "A nullable JSON object (structured key-value data) that may fail to parse, returning a String error."

### Usage Example

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ToolUseEvent {
    tool_use: OptionalJsonObjectResult,  // May be null, may be malformed JSON
}
```

### Impact

**Reduces complexity score from:** ~260  
**Reduces to:** 1 (type alias reference)  
**Affects:** 1 field (`tool_use`)

---

## Summary of All Type Aliases

### Core Reusable Aliases (Recommended for Base Module)

```rust
// Pattern 1: Timestamps
pub type OptionalUtcDateTimeResult = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;

// Pattern 3: JSON objects
pub type OptionalJsonObjectResult = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
```

### Domain-Specific Aliases (Configuration Modules)

```rust
// Pattern 2: Pricing configurations
pub type PricingAdaptersMapResult = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValue>, String>;
pub type PricingAdapterModelsMapResult = Result<::std::collections::HashMap<String, HoopConfigPricingAdaptersValueModelsValue>, String>;
pub type GenericPricingAdaptersMapResult = Result<::std::collections::HashMap<String, PricingConfigAdaptersValue>, String>;
pub type GenericPricingAdapterModelsMapResult = Result<::std::collections::HashMap<String, PricingConfigAdaptersValueModelsValue>, String>;

// Pattern 2: Stuck detector configurations
pub type StuckDetectorAdaptersMapResult = Result<::std::collections::HashMap<String, HoopConfigStuckDetectorAdaptersValue>, String>;
```

---

## Implementation Guidance

### Where to Define These Aliases

**Option A: In `hoop-schema` (Recommended)**

Add to `hoop-schema/src/lib.rs` or a new `hoop-schema/src/types.rs`:

```rust
// Re-export commonly-used type aliases for generated code
pub mod types {
    pub use super::generated_types::OptionalUtcDateTimeResult;
    pub use super::generated_types::OptionalJsonObjectResult;
    // ... domain-specific aliases
}
```

**Option B: Via Build Script Injection**

Modify `hoop-schema/build.rs` to prepend aliases to the generated file:

```rust
fn main() {
    let aliases = r#"
// Auto-generated type aliases for complexity reduction
type OptionalUtcDateTimeResult = Result<Option<chrono::DateTime<chrono::offset::Utc>>, String>;
type OptionalJsonObjectResult = Result<Option<::serde_json::Map<String, ::serde_json::Value>>, String>;
"#;

    // Prepend to generated types.rs
}
```

### Naming Convention Checklist

Each alias follows this convention:
- ✅ **PascalCase** - Rust type naming standard
- ✅ **Descriptive domain prefix** - What domain? (Pricing, StuckDetector, etc.)
- ✅ **Semantic payload** - What does it contain? (DateTime, JsonObject, Adapters)
- ✅ **Structure suffix** - What container? (Result, Map)
- ✅ **No abbreviations** - "UtcDateTime" not "UTCDT", "JsonObject" not "JSONObject"
- ✅ **Consistent ordering** - Optional → Payload → Result

### Complexity Reduction Impact

| Pattern | Instances | Original Complexity | After Alias | Reduction |
|---------|-----------|---------------------|-------------|-----------|
| DateTime timestamps | 27 | ~250 each | 1 | ~6,723 total |
| Adapter maps | 5 | ~200-250 each | 1 | ~1,125 total |
| JSON objects | 1 | ~260 | 1 | 260 total |
| **TOTAL** | **33** | **~8,108** | **33** | **~8,075** |

---

## Alternative Approach: Clippy Threshold Adjustment

If implementing type aliases is not feasible, an alternative is to increase Clippy's type complexity threshold in `clippy.toml`:

```toml
[type-complexity]
threshold = 300  # Increase from default 250 to accommodate generated patterns
```

**Trade-off:** This suppresses warnings without improving code readability. Type aliases are preferred for maintainability.

---

## Next Steps

1. **Review this catalog** with the HOOP team to confirm alias naming
2. **Implement aliases** in `hoop-schema` at the generation level
3. **Verify fix** by re-running `cargo clippy -- -W type_complexity`
4. **Update documentation** to reference these aliases in schema contribution guides
5. **Consider schema improvements** to reduce complexity at the source (e.g., defining custom timestamp types in JSON Schema)

---

## Related Files

- `docs/type-complexity-parsed.md` — Source warning analysis
- `docs/type-complexity-raw.txt` — Original clippy output
- `clippy.toml` — Current Clippy configuration
- `hoop-schema/build.rs` — Schema code generation script
- `hoop-schema/build/out/types.rs` — Generated types (DO NOT EDIT MANUALLY)
