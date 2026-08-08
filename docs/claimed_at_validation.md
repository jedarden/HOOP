# claimed_at Timestamp Validation and Transformation Rules

## Overview

The `claimed_at` field stores when a bead was claimed by a worker in the NEEDLE fleet. This timestamp originates from external NEEDLE events written to `events.jsonl` files and is stored in the HOOP SQLite database's `collision_index` table.

## Validation Function

**Location**: `hoop-daemon/src/supervisor.rs` (lines 1155-1173)

The single validation point is the `sanitize_timestamp()` function:

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

## Validation Rules

### Required Format
- **Standard**: RFC3339 timestamp format
- **Examples of valid formats**:
  - `2026-04-21T18:42:10Z` (UTC with Z suffix)
  - `2026-04-21T18:42:10+00:00` (UTC with offset)
  - `2026-04-21T18:42:10.123Z` (with milliseconds)
  - `2026-04-21T18:42:10+05:30` (with timezone offset)

### Invalid Formats
The following are **invalid** and trigger fallback to current time:
- Empty strings (`""`)
- Partial timestamps (`"2026-04-21"` - missing time component)
- Wrong formats (`"April 21, 2026"`)
- Garbage strings (`"not-a-timestamp"`)
- Timestamps with whitespace (`" 2026-04-21T18:42:10Z"`)
- Invalid timezone offsets (outside ±23:59)
- Out-of-range date values (month 0, day 32, hour 24, etc.)

### Date Range Constraints
- **No explicit range validation**: The RFC3339 parser accepts dates from year 1 AD to year 9999
- **Negative timestamps**: Dates before Unix epoch (1970) are accepted
- **Future dates**: No restriction on future dates
- **Leap seconds**: Accepted (e.g., `2016-12-31T23:59:60Z`)

## Error Handling

### Empty Timestamps
- **Detection**: `ts.is_empty()` check
- **Action**: Log warning and fallback to current time
- **Log message**: `"Empty timestamp in event, using current time as fallback"`

### Invalid Timestamps
- **Detection**: `DateTime::parse_from_rfc3339()` returns `Err`
- **Action**: Log warning with error details and fallback to current time
- **Log message**: `"Invalid timestamp format '{ts}' in event: {error}, using current time as fallback"`
- **Common error messages**:
  - `"premature end of input"` (empty string)
  - `"invalid timestamp"` (malformed)
  - `"unexpected character"` (invalid characters)

### No Crashes
- Invalid timestamps **never cause panics**
- Always produce a valid RFC3339 string (either original or current time)
- System continues processing events even with malformed input

## Transformations

### No Transformations Applied
When a timestamp is **valid**, the function applies **no transformations**:
- ✗ No timezone conversion
- ✗ No normalization
- ✗ No truncation
- ✓ Preserves original format exactly (including fractional seconds, offset format, etc.)

### Fallback Value
When a timestamp is **invalid**, the transformation is:
- **New value**: `Utc::now().to_rfc3339()`
- **Format**: Current UTC time in RFC3339 format
- **Precision**: Microsecond precision (6 decimal places)
- **Example**: `2026-08-08T14:23:45.123456Z`

## Data Flow

### Write Path (Event Processing)
1. **Source**: NEEDLE writes Claim event to `events.jsonl` with raw `claimed_at` string
2. **Extraction**: HOOP supervisor reads event, extracts timestamp string
3. **Validation**: `sanitize_timestamp()` validates the string
4. **Storage**: Validated (or fallback) timestamp written to SQLite

```rust
// In supervisor.rs, line 1172
let claimed_at = sanitize_timestamp(ts);
```

### Storage Schema
**Table**: `collision_index`  
**Column**: `claimed_at TEXT NOT NULL`  
**Row**: `(bead_id, project, worker, claimed_at, file_paths, updated_at)`

### Read Path (Retrieval)
- **No parsing on read**: Retrieved as-is from SQLite
- **No validation**: String returned directly to caller
- **Usage**: Caller responsible for parsing if needed

```rust
// In fleet.rs, line 5658
claimed_at: row.get(3)?,  // Retrieved as-is, no parsing
```

## Default Values and Fallback Logic

### Primary Fallback
- **Trigger**: Empty string or parse failure
- **Value**: `Utc::now().to_rfc3339()`
- **Rationale**: Ensures database always contains valid timestamp
- **Side effect**: Log warning emitted

### No Default on Missing Field
- The `claimed_at` field is **always present** in Claim events
- SQLite column is `NOT NULL`
- No NULL handling needed

## Security Considerations

### SQL Injection Protection
- **Parameterized queries**: All SQLite operations use parameterized queries
- **String storage**: Invalid strings (including SQL injection attempts) stored harmlessly as TEXT
- **Tested cases**: SQL injection patterns validated in test suite
- **No code execution**: Invalid timestamps never interpreted as SQL

### Tested Attack Patterns
The test suite validates that these patterns are safely handled:
- `'; DROP TABLE collision_index; --`
- `' OR '1'='1`
- `' UNION SELECT * FROM collision_index --`
- And 8 other SQL injection patterns

All are rejected as invalid timestamps and fall back to current time.

## Test Coverage

**Location**: `hoop-daemon/tests/claimed_at_parsing.rs` (750+ lines)

### Test Categories
1. **Valid formats** (11 tests): Standard RFC3339 variants, offsets, fractional seconds
2. **Invalid formats** (4 tests): Empty, partial, wrong format, garbage
3. **Edge cases** (15 tests): Whitespace, case sensitivity, special characters, boundary values
4. **Security** (1 test): SQL injection attempts
5. **Round-trip** (3 tests): Verify timestamps survive storage/retrieval
6. **Timezone offsets** (2 tests): Various offset formats
7. **Fractional precision** (2 tests): 0-9 decimal places
8. **Extreme dates** (2 tests): Year 1 AD, year 9999
9. **Leap seconds** (1 test): Second value of 60
10. **Empty variants** (1 test): Empty string and whitespace variants

### Running Tests
```bash
# Run only claimed_at parsing tests
cargo test claimed_at_parsing

# Run with output
cargo test claimed_at_parsing -- --nocapture

# Specific test
cargo test empty_timestamp_is_invalid
```

## Logging

All validation failures are logged at **WARNING level**:

```log
WARN Empty timestamp in event, using current time as fallback
WARN Invalid timestamp format '' in event: premature end of input, using current time as fallback
WARN Invalid timestamp format '2026-04-21' in event: invalid timestamp, using current time as fallback
```

## Related Documentation

- **Event processing**: `hoop-daemon/src/supervisor.rs` (event loop)
- **Storage schema**: `hoop-daemon/src/fleet.rs` (collision_index table)
- **Test suite**: `hoop-daemon/tests/claimed_at_parsing.rs` (comprehensive tests)
- **Investigation notes**: 
  - `notes/bf-5wxp7-claimed_at-investigation.md`
  - `notes/bf-25ihb-claimed_at-parsing-investigation.md`

## Summary

| Aspect | Rule |
|--------|------|
| **Format** | RFC3339 timestamp string |
| **Empty handling** | Fallback to current time (logged) |
| **Invalid handling** | Fallback to current time (logged) |
| **Timezone conversion** | None (preserves original) |
| **Normalization** | None (preserves original) |
| **Truncation** | None (preserves original) |
| **Date range** | Year 1 AD to 9999 (RFC3339 limits) |
| **Future dates** | Allowed (no validation) |
| **Storage type** | TEXT NOT NULL in SQLite |
| **Read parsing** | None (retrieved as string) |
| **Security** | Parameterized queries, safe from SQL injection |
