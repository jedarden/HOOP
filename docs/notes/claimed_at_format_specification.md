# `claimed_at` Timestamp Format Specification

**Date:** 2026-08-21  
**Bead:** hoop-b6338b68  
**Status:** EXTRACTED AND DOCUMENTED

---

## Expected Format

**`claimed_at` timestamps MUST be in RFC3339 format (ISO 8601).**

### Valid Examples

```rust
// Basic RFC3339 (UTC)
"2026-04-21T18:42:10Z"

// With milliseconds
"2026-04-21T18:42:10.123Z"

// With timezone offset
"2026-04-21T18:42:10+00:00"
"2026-04-21T18:42:10+05:30"
"2026-04-21T18:42:10-08:00"

// With fractional seconds (micro/nanoseconds)
"2026-04-21T18:42:10.123456Z"
"2026-04-21T18:42:10.123456789Z"
```

### Invalid Examples

```rust
// Empty string (causes "premature end of input" error)
""

// Partial timestamp (missing time component)
"2026-04-21"

// Wrong format
"April 21, 2026"

// Garbage
"not-a-timestamp"
```

---

## Files and Functions Handling `claimed_at`

### 1. **`hoop-daemon/src/supervisor.rs`** (Lines 1084-1112)

**Function:** `sanitize_timestamp(ts: &str) -> String`

**Purpose:** Validates and sanitizes timestamp strings from events.jsonl before storage.

**Behavior:**
- Returns the original timestamp if it's valid RFC3339
- Returns `Utc::now().to_rfc3339()` if:
  - Input is empty string
  - Input is not valid RFC3339 format
- Logs a warning when falling back to current time

**Code:**
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

**Usage in `update_fleet_from_event()` (Lines 1177-1203):**
```rust
NeedleEvent::Claim { .. } => {
    if let Some(ref proj) = project {
        let now = chrono::Utc::now().to_rfc3339();
        // Sanitize the timestamp to handle empty/invalid values
        let claimed_at = sanitize_timestamp(ts);
        let claimed_at_opt = if claimed_at.is_empty() {
            None
        } else {
            Some(claimed_at)
        };
        let entry = fleet::CollisionIndexEntry {
            bead_id: bead_id.to_string(),
            project: proj.clone(),
            worker: Some(worker.to_string()),
            claimed_at: claimed_at_opt,  // Option<String>
            file_paths: vec![],
            updated_at: now,
        };
        // ... upsert to database
    }
}
```

---

### 2. **`hoop-daemon/src/fleet.rs`** (Lines 7724-7734, 8286-8308)

**Struct:** `CollisionIndexEntry`

**Definition:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionIndexEntry {
    pub bead_id: String,
    pub project: String,
    pub worker: Option<String>,
    /// The timestamp when this bead was claimed. May be absent if the field
    /// was missing from the source data or if the bead was never claimed.
    pub claimed_at: Option<String>,  // <-- RFC3339 timestamp or None
    pub file_paths: Vec<String>,
    pub updated_at: String,
}
```

**Function:** `upsert_collision_entry_conn()` (Lines 8286-8308)

**Purpose:** Writes `CollisionIndexEntry` to the `collision_index` table in SQLite database.

**Behavior:**
- Stores `claimed_at` as-is (validated upstream by `sanitize_timestamp()`)
- Uses `INSERT OR REPLACE` / `ON CONFLICT` to handle duplicates
- `claimed_at` can be `NULL` in database (None in Rust)

**SQL Schema (Line 4987-4999):**
```sql
CREATE TABLE IF NOT EXISTS collision_index (
    bead_id    TEXT PRIMARY KEY NOT NULL,
    project    TEXT NOT NULL,
    worker     TEXT,
    claimed_at TEXT,              -- <-- Can be NULL
    file_paths TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL
)
```

---

## Validation Rules and Constraints

### At Write Time (Events → Database)

1. **Empty strings → Fallback to current time**
   - Detected by `sanitize_timestamp()`
   - Logged as warning
   - Replaced with `Utc::now().to_rfc3339()`

2. **Invalid RFC3339 format → Fallback to current time**
   - Detected by `DateTime::parse_from_rfc3339()`
   - Logged as warning with error details
   - Replaced with `Utc::now().to_rfc3339()`

3. **Valid RFC3339 → Stored as-is**
   - No transformation applied
   - Original string preserved in database

### At Read Time (Database → Application)

1. **`None` values handled gracefully**
   - `claimed_at: Option<String>` can be `None`
   - Represents "never claimed" or "missing field"
   - No parsing errors on `None`

2. **Invalid formats in existing data**
   - Will fail `DateTime::parse_from_rfc3339()` if read
   - Should not occur after `sanitize_timestamp()` was applied
   - If encountered, treat as data quality issue

---

## Type Summary

| Context | Type | Nullable | Validation |
|---------|------|----------|------------|
| **events.jsonl** | `String` | Empty string allowed | Sanitized before DB write |
| **Rust struct** | `Option<String>` | Yes (None allowed) | RFC3339 if Some() |
| **SQLite** | `TEXT` | Yes (NULL allowed) | No DB-level constraints |
| **After sanitization** | `String` | No (never empty) | Always valid RFC3339 |

---

## Parsing Library

**Chrono crate:** `chrono::DateTime::parse_from_rfc3339()`

- Accepts full RFC3339 specification
- Supports:
  - UTC: `T18:42:10Z`
  - Offsets: `+00:00`, `+05:30`, `-08:00`
  - Fractional seconds: `.123`, `.123456`, `.123456789`
- Rejects:
  - Empty strings (premature end of input)
  - Partial timestamps (date only, time only)
  - Non-ISO formats

---

## Test Coverage

**Test file:** `hoop-daemon/tests/claimed_at_parsing.rs`

**Validates:**
- ✅ Valid RFC3339 timestamps parse correctly
- ✅ Milliseconds/microseconds/nanoseconds supported
- ✅ Timezone offsets supported
- ✅ Empty strings handled gracefully
- ✅ Invalid formats detected without panic
- ✅ `None` values round-trip correctly
- ✅ Database operations succeed with all variants

---

## Summary

**`claimed_at` is an RFC3339 timestamp string (ISO 8601 format) stored as `Option<String>` in Rust and `TEXT` (nullable) in SQLite.**

- **Source:** Extracted from events.jsonl `Claim` events
- **Validation:** `sanitize_timestamp()` in `supervisor.rs` ensures RFC3339 compliance
- **Storage:** `collision_index` table in `~/.hoop/fleet.db`
- **Graceful degradation:** Empty/invalid values fall back to current time with warning logs
- **Tested:** Comprehensive test suite in `claimed_at_parsing.rs`

---

## Related Documentation

- **RFC3339 Specification:** [RFC 3339 - Date and Time on the Internet](https://www.rfc-editor.org/rfc/rfc3339)
- **Chrono Documentation:** [chrono::DateTime::parse_from_rfc3339](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339)
- **HOOP Plan:** `docs/plan/plan.md` (Section on event processing and collision detection)
