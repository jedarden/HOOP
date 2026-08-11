# claimed_at Timestamp Parsing Error Investigation

**Bead ID:** bf-5wxp7  
**Date:** 2026-08-11  
**Parent:** bf-8ukbk (Fix claimed_at timestamp format parsing)

## Executive Summary

The `claimed_at` timestamp parsing error occurs in the **br CLI** (bead-forge) codebase, not HOOP. The error happens when empty strings or invalid timestamp formats are encountered in the `worker_sessions.claimed_at` field, causing "Invalid claimed_at format: premature end of input" errors during bead close operations.

## 1. Location of Parsing Logic

**File:** `/home/coding/bead-forge/src/velocity.rs`  
**Lines:** 26-46  
**Function:** `parse_datetime(s: &str) -> Result<DateTime<Utc>>`

### Parsing Function Code

```rust
fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    use chrono::NaiveDateTime;
    let t = s.trim();
    // Early reject of empty strings
    if t.is_empty() {
        return Err(anyhow::anyhow!("Invalid claimed_at format: empty string"));
    }
    // Try RFC3339 first
    match DateTime::parse_from_rfc3339(t) {
        Ok(dt) => Ok(dt.with_timezone(&Utc)),
        Err(_) => {
            // SQLite-native format fallback
            for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
                if let Ok(ndt) = NaiveDateTime::parse_from_str(t, fmt) {
                    return Ok(ndt.and_utc());
                }
            }
            Err(anyhow::anyhow!("Invalid claimed_at format: {}", t))
        }
    }
}
```

### Called From

**Function:** `update_session_on_close()` at line 132  
**Context:** Updates worker session when a bead is closed, calculates duration, and updates velocity statistics

## 2. Current Timestamp Format Expected vs. Received

### Expected Formats (Accepted)

1. **RFC3339 with timezone** (primary format)
   - `2026-05-15T21:10:36+00:00`
   - `2026-05-15T21:10:36Z`
   - `2026-04-21T18:42:10.123Z` (with milliseconds)
   - `2026-04-21T18:42:10+05:30` (with offset)

2. **SQLite native format** (legacy fallback)
   - `2026-05-15 21:10:36` (space separator)
   - `2026-05-15T21:10:36` (T separator)

### Invalid Formats (Cause Errors)

1. **Empty string** - `"":`
   - Error: "Invalid claimed_at format: empty string"
   - Root cause: Should be NULL in database, not empty string

2. **Partial date** - Missing time component
   - `2026-04-21`
   - Error: "Invalid claimed_at format: 2026-04-21"

3. **Wrong format**
   - `April 21, 2026`
   - Error: "Invalid claimed_at format: April 21, 2026"

4. **Garbage**
   - `not-a-timestamp`
   - Error: "Invalid claimed_at format: not-a-timestamp"

## 3. Why Parsing is Failing

### Root Cause Analysis

The parsing fails due to **format mismatch** between what's stored in the database and what the parser expects:

1. **Historical data issue**: Early versions of the br CLI's claim.rs omitted `claimed_at` from the INSERT column list, causing SQLite to fall back to its non-RFC3339 `CURRENT_TIMESTAMP` default format (`YYYY-MM-DD HH:MM:SS`)

2. **Empty string corruption**: Some rows have empty strings instead of NULL values or valid timestamps

3. **Legacy format support**: The parser tries to support both RFC3339 and SQLite-native formats, but not all edge cases

### Error Messages

Two distinct error messages from `parse_datetime()`:

1. Line 31: `"Invalid claimed_at format: empty string"` - for empty strings after trim
2. Line 43: `"Invalid claimed_at format: {t}"` - for unparseable non-empty strings

## 4. Format of Other Timestamp Fields

### In worker_sessions table (same table as claimed_at)

- **closed_at DATETIME** - Optional, same format expectations as claimed_at
- **created_at DATETIME** - In other tables (see below)

### In other bead-forge tables

All use DATETIME with DEFAULT CURRENT_TIMESTAMP:

```sql
-- issues table
created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
closed_at DATETIME
due_at DATETIME
defer_until DATETIME
deleted_at DATETIME

-- events table
created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP

-- export_hashes table
exported_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP

-- velocity_stats table
last_updated DATETIME
```

### In HOOP's CollisionIndexEntry

```rust
pub struct CollisionIndexEntry {
    pub bead_id: String,
    pub project: String,
    pub worker: Option<String>,
    pub claimed_at: String,  // Stored as TEXT, not enforced
    pub file_paths: Vec<String>,
    pub updated_at: String,
}
```

**Note:** HOOP stores timestamps as TEXT without database-level validation, allowing invalid formats to persist until parsing time.

## 5. Dependencies

- **chrono crate**: `DateTime`, `NaiveDateTime`, `Utc`
- **anyhow crate**: Error handling
- **rusqlite**: Database access

## 6. Test Coverage

**File:** `/home/coding/HOOP/hoop-daemon/tests/claimed_at_parsing.rs`

Comprehensive test suite (750+ lines) covering:

- Valid RFC3339 formats (with/without milliseconds, offsets)
- Invalid formats (empty, partial, wrong format, garbage)
- Edge cases (whitespace, case sensitivity, special characters)
- Security tests (SQL injection attempts)
- Round-trip preservation tests
- Boundary values (invalid dates, extreme dates, leap seconds)

## 7. Recommendations for Fix

1. **Validate at write time**: Add format validation in claim.rs when creating worker_sessions rows
2. **Use NULL instead of empty**: Default to NULL for missing claimed_at values
3. **Migration**: Scan and repair existing empty/invalid claimed_at values
4. **Graceful degradation**: Current approach (skip velocity tracking vs. error entire close) is good
5. **Add tests**: Ensure fix covers the edge cases in the test suite

## 8. Related Beads

- **bf-8ukbk** (parent): Fix claimed_at timestamp format parsing
- **bf-3ebyx** (dependency): Locate claimed_at parsing code in br CLI - CLOSED
- **bf-3fwnl** (dependency): Verify no regressions in timestamp parsing
- **bf-5i1ln** (original failure): Bead that failed to close with the error

## 9. Database Schema Context

```sql
CREATE TABLE IF NOT EXISTS worker_sessions (
    worker_id TEXT NOT NULL,
    model TEXT,
    harness TEXT,
    harness_version TEXT,
    claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bead_id TEXT REFERENCES issues(id) ON DELETE SET NULL,
    workspace_path TEXT NOT NULL,
    closed_at DATETIME,
    duration_seconds INTEGER,
    PRIMARY KEY (worker_id, claimed_at)
);
```

**Note:** `claimed_at` is NOT NULL with DEFAULT CURRENT_TIMESTAMP, but legacy data may have empty strings or non-standard formats.

## 10. Verification Steps

To verify the fix works correctly:

1. Run the comprehensive test suite: `cargo test --package hoop-daemon claimed_at_parsing`
2. Test with real data containing edge cases
3. Verify no regressions in other timestamp parsing
4. Check that velocity stats compute correctly for valid timestamps
5. Confirm empty/invalid timestamps are handled gracefully without crashing close operations

---

**Investigation complete** - All findings documented to guide fix implementation.
