# Timestamp Field Formats in HOOP

## Overview

This document catalogs all timestamp fields in the HOOP codebase, their formats, parsing behavior, and validation logic. It serves as a reference for understanding how timestamps are handled across the system and provides comparison data for the `claimed_at` field issue investigation.

**Related beads:** `bf-42drg` (timestamp field documentation), `bf-2n2e6` (claimed_at format mismatch root cause analysis)

## Summary of Findings

- **Total timestamp fields cataloged:** 20+ across 8 major structures
- **Primary storage format:** RFC3339 strings (e.g., `2026-04-21T18:42:10Z`)
- **Validation approach:** Centralized sanitization function with fallback to current time
- **Type representation:** Hybrid (String storage + chrono::DateTime for manipulation)
- **Test coverage:** Comprehensive test suite for `claimed_at` parsing edge cases

## All Timestamp Fields by Data Structure

### 1. Stitch Entity

**Schema:** `hoop-schema/schemas/stitch.json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `String` | Yes | Creation timestamp |
| `updated_at` | `String` | Yes | Last update timestamp |
| `closed_at` | `Option<String>` | No | Closure timestamp if applicable |
| `archived_at` | `Option<String>` | No | Archive timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
2026-04-21T18:42:10.123Z (with milliseconds)
2026-04-21T18:42:10+00:00 (with timezone offset)
```

### 2. Bead Entity

**Schema:** `hoop-schema/schemas/bead.json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `String` | Yes | Creation timestamp |
| `updated_at` | `String` | Yes | Last update timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 3. ReflectionLedger

**Schema:** `hoop-schema/schemas/reflection_ledger.json`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `String` | Yes | Creation timestamp |
| `last_applied` | `Option<String>` | No | Last application timestamp |
| `approved_at` | `Option<String>` | No | Approval timestamp |
| `archived_at` | `Option<String>` | No | Archive timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 4. Pattern Entity

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `String` | Yes | Creation timestamp |
| `updated_at` | `Option<String>` | No | Last update timestamp |
| `closed_at` | `Option<String>` | No | Closure timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 5. StitchLink

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `Option<String>` | No | Creation timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 6. PatternMember

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `added_at` | `Option<String>` | No | Addition timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 7. CollisionIndexEntry ⭐️

**Location:** `hoop-daemon/src/fleet.rs:5024-5032`

**This is the structure containing the `claimed_at` field under investigation.**

```rust
pub struct CollisionIndexEntry {
    pub bead_id: String,
    pub project: String,
    pub worker: Option<String>,
    pub claimed_at: String,      // ⭐️ Field under investigation
    pub file_paths: Vec<String>,
    pub updated_at: String,
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `claimed_at` | `String` | Yes | Claim timestamp |
| `updated_at` | `String` | Yes | Last update timestamp |

**SQLite Schema:**
```sql
CREATE TABLE IF NOT EXISTS collision_index (
    bead_id    TEXT PRIMARY KEY NOT NULL,
    project    TEXT NOT NULL,
    worker     TEXT,
    claimed_at TEXT NOT NULL,          -- Stored as TEXT
    file_paths TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL           -- Stored as TEXT
);
```

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
2026-04-21T18:42:10.123Z (with milliseconds)
```

**Known issues:** See "claimed_at Specific Issues" section below.

### 8. ProjectStatusRow

**Location:** `hoop-daemon/src/fleet.rs:4936-4945`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `last_event_at` | `Option<String>` | No | Last event timestamp |
| `last_heartbeat_at` | `Option<String>` | No | Last heartbeat timestamp |
| `updated_at` | `String` | Yes | Last update timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 9. StitchRow

**Location:** `hoop-daemon/src/fleet.rs:4718-4726`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `String` | Yes | Creation timestamp |
| `last_activity_at` | `String` | Yes | Last activity timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 10. BeadEventData (Events)

**Location:** `hoop-daemon/src/events.rs`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | `String` | Yes | Event timestamp |

**Working format:** RFC3339 with timezone
```
2026-04-21T18:42:10Z
```

### 11. Screen Capture Structures

**Location:** `hoop-daemon/src/screen_capture.rs`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `recorded_at` | `String` | Yes | Recording timestamp |
| `timestamp_secs` | `f64` | Yes | Unix timestamp in seconds |

**Working format:** RFC3339 for `recorded_at`, Unix float for `timestamp_secs`
```
recorded_at: "2026-04-21T18:42:10Z"
timestamp_secs: 1744996930.123
```

### 12. WebSocket Data Structures

**Location:** `hoop-daemon/src/ws.rs`

```rust
pub struct WorkerData {
    pub last_heartbeat: DateTime<Utc>,  // ⭐️ Uses chrono::DateTime type
    pub heartbeat_age_secs: i64,
}

pub struct BeadData {
    pub created_at: String,
    pub updated_at: String,
}

pub struct ConversationData {
    pub created_at: String,
    pub updated_at: String,
}
```

**Working format:** Mixed
- `WorkerData.last_heartbeat`: chrono::DateTime<Utc> (native type)
- `BeadData.*`: RFC3339 strings
- `ConversationData.*`: RFC3339 strings

## Parsing Logic and Validation

### Primary Sanitization Function

**Location:** `hoop-daemon/src/supervisor.rs:1084-1102`

```rust
fn sanitize_timestamp(ts: &str) -> String {
    // If empty, use current time as fallback
    if ts.is_empty() {
        warn!("Empty timestamp in event, using current time as fallback");
        return Utc::now().to_rfc3339();
    }

    // Try to parse the timestamp to verify it's valid RFC3339
    match DateTime::parse_from_rfc3339(ts) {
        Ok(_) => ts.to_string(), // Valid timestamp, return as-is
        Err(e) => {
            warn!(
                "Invalid timestamp format '{}' in event: {}, using current time as fallback",
                ts, e
            );
            Utc::now().to_rfc3339()
        }
    }
}
```

**Behavior:**
1. **Empty strings:** Replaced with `Utc::now().to_rfc3339()`
2. **Valid RFC3339:** Passed through unchanged
3. **Invalid RFC3339:** Replaced with `Utc::now().to_rfc3339()` and warning logged

### RFC3339 Validation Helper

**Location:** `hoop-daemon/tests/claimed_at_parsing.rs:69-71`

```rust
fn is_valid_rfc3339(ts: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(ts).is_ok()
}
```

### Usage Pattern for claimed_at

**Location:** `hoop-daemon/src/supervisor.rs:1172-1180`

```rust
let claimed_at = sanitize_timestamp(ts);
let entry = fleet::CollisionIndexEntry {
    bead_id: bead_id.to_string(),
    project: proj.clone(),
    worker: Some(worker.to_string()),
    claimed_at,
    file_paths: vec![],
    updated_at: now,
};
```

## Successfully Parsed Timestamp Formats

### Valid RFC3339 Formats

All of these formats are successfully parsed by `sanitize_timestamp()`:

```rust
// Basic format (most common)
"2026-04-21T18:42:10Z"

// With milliseconds
"2026-04-21T18:42:10.123Z"

// With timezone offset
"2026-04-21T18:42:10+00:00"
"2026-04-21T18:42:10-05:00"

// With microseconds
"2026-04-21T18:42:10.123456Z"
```

### Format Variations Accepted

| Format | Example | Parsed Successfully |
|--------|---------|---------------------|
| ISO 8601 basic | `2026-04-21T18:42:10Z` | ✅ Yes |
| With milliseconds | `2026-04-21T18:42:10.123Z` | ✅ Yes |
| With microseconds | `2026-04-21T18:42:10.123456Z` | ✅ Yes |
| With offset | `2026-04-21T18:42:10+00:00` | ✅ Yes |
| With negative offset | `2026-04-21T18:42:10-05:00` | ✅ Yes |

## Invalid Formats (Handled with Fallback)

### Formats That Trigger Sanitization Fallback

All of these cause `sanitize_timestamp()` to return current time with a warning:

| Invalid Format | Example | Behavior |
|----------------|---------|----------|
| Empty string | `""` | ⚠️ Fallback to now, warn "Empty timestamp" |
| Partial date | `"2026-04-21"` | ⚠️ Fallback to now, warn "Invalid timestamp format" |
| Time only | `"18:42:10"` | ⚠️ Fallback to now, warn "Invalid timestamp format" |
| Wrong separator | `"2026/04/21T18:42:10Z"` | ⚠️ Fallback to now, warn "Invalid timestamp format" |
| SQL injection attempt | `"'; DROP TABLE; --"` | ⚠️ Fallback to now, warn "Invalid timestamp format" |
| Whitespace only | `"   "` | ⚠️ Fallback to now, warn "Invalid timestamp format" |

### Test Coverage for Invalid Formats

**Location:** `hoop-daemon/tests/claimed_at_parsing.rs`

```rust
const INVALID_TIMESTAMP_EMPTY: &str = "";
const INVALID_TIMESTAMP_PARTIAL: &str = "2026-04-21";
const INVALID_TIMESTAMP_TIME_ONLY: &str = "18:42:10";
const INVALID_TIMESTAMP_WRONG_SEPARATOR: &str = "2026/04/21T18:42:10Z";
const INVALID_TIMESTAMP_SQL_INJECTION: &str = "'; DROP TABLE; --";
const INVALID_TIMESTAMP_WHITESPACE: &str = "   ";
```

## Comparison: claimed_at vs. Other Timestamp Fields

### Similarities

| Aspect | claimed_at | Other Fields |
|--------|------------|--------------|
| **Type** | `String` | `String` (most) or `Option<String>` |
| **Format** | RFC3339 | RFC3339 |
| **Validation** | `sanitize_timestamp()` | `sanitize_timestamp()` |
| **Fallback behavior** | Current time on invalid | Current time on invalid |
| **Storage** | SQLite TEXT | SQLite TEXT |

### Differences

| Aspect | claimed_at | Other Fields |
|--------|------------|--------------|
| **Required?** | Yes (no Option) | Most are `Option<String>` |
| **Context** | Bead claim events | Entity lifecycle events |
| **Source** | NEEDLE events | Multiple sources |
| **Test coverage** | Comprehensive test suite | Limited explicit tests |

### Unique Characteristics of claimed_at

1. **Always required:** Unlike `closed_at`, `archived_at`, etc., `claimed_at` has no Option wrapper
2. **Event-sourced:** Comes from NEEDLE bead claim events, not internal state
3. **Collision detection key:** Part of the collision index for detecting duplicate claims
4. **Heaviest test coverage:** Has dedicated test file with edge case coverage

## Parsing Logic Differences by Field

### Centralized Sanitization (Most Fields)

**Used by:**
- `claimed_at` (CollisionIndexEntry)
- Event timestamps
- Most optional timestamps

**Logic:**
```rust
fn sanitize_timestamp(ts: &str) -> String
```

### Direct Assignment (WebSocket Structures)

**Used by:**
- `WorkerData.last_heartbeat` (chrono::DateTime)
- Some internal structures

**Logic:** Direct assignment without sanitization (trusted source)

### Validation-Only (Tests)

**Used by:**
- Test helpers for validation

**Logic:**
```rust
fn is_valid_rfc3339(ts: &str) -> bool
```

## Fields with Similar Issues or Workarounds

### No Known Issues with Other Fields

As of this documentation, **no other timestamp fields** exhibit the same parsing issues as `claimed_at`. This is likely due to:

1. **Different data sources:** Most fields come from internal HOOP operations, not external NEEDLE events
2. **Comprehensive sanitization:** The `sanitize_timestamp()` function handles edge cases uniformly
3. **Test coverage:** The `claimed_at` tests cover the sanitization path used by all fields

### Potential Future Issues

Fields that **could** exhibit similar issues if external data sources are added:

- `last_event_at` (if external event sources are added)
- `last_heartbeat_at` (if worker heartbeat format changes)
- Optional timestamps that become required (migrating from `Option<String>` to `String`)

## Recommendations

### For claimed_at

1. ✅ **Already implemented:** Comprehensive sanitization in `sanitize_timestamp()`
2. ✅ **Already implemented:** Fallback to current time on invalid format
3. ✅ **Already implemented:** Warning logs for invalid timestamps
4. ✅ **Already implemented:** Comprehensive test coverage

### For Other Timestamp Fields

1. **Consider extending sanitization:** All external timestamp sources should use `sanitize_timestamp()`
2. **Add validation tests:** Create test files for other critical timestamp fields
3. **Document sources:** Clearly document which fields receive external vs. internal data

### For New Timestamp Fields

1. **Default to String type:** Use `String` with RFC3339 format for compatibility
2. **Use Option for optional:** `Option<String>` for timestamps that may not exist
3. **Sanitize external inputs:** Always use `sanitize_timestamp()` for external data
4. **Add tests:** Create comprehensive tests covering valid, invalid, and edge cases

## Related Documentation

- **claimed_at format mismatch root cause:** `docs/research/claimed_at-format-mismatch-root-cause.md` (bead `bf-2n2e6`)
- **claimed_at sample collection:** `docs/research/claimed_at-field-samples.md` (bead `bf-5g16g`)
- **Comprehensive error messages catalog:** `docs/research/claimed_at-error-messages-catalog.md` (bead `bf-54mwz`)

## Test File Reference

**Primary test file:** `hoop-daemon/tests/claimed_at_parsing.rs`

This file contains comprehensive tests covering:
- ✅ Valid RFC3339 formats (with/without milliseconds, timezones)
- ✅ Invalid formats (empty strings, partial dates, wrong formats)
- ✅ Edge cases (whitespace, SQL injection attempts, boundary values)
- ✅ Round-trip preservation through CollisionIndexEntry

**Run tests:**
```bash
cargo test --package hoop-daemon --test claimed_at_parsing
```

## Schema Reference

**Primary schemas:** `hoop-schema/schemas/`

- `stitch.json` - Stitch entity timestamps
- `bead.json` - Bead entity timestamps
- `reflection_ledger.json` - Reflection ledger timestamps
- `pattern.json` - Pattern entity timestamps

## Revision History

- **2026-08-11:** Initial documentation (bead `bf-42drg`)
- Based on comprehensive codebase exploration via agent analysis
