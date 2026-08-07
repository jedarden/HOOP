# claimed_at Timestamp Format Analysis

**Generated:** 2026-08-07  
**Bead:** bf-61v20  
**Purpose:** Document current `claimed_at` timestamp format behavior

## Executive Summary

The `claimed_at` field has a **critical format mismatch** between what HOOP expects (RFC3339) and what it actually receives from the beads_rust/br CLI (SQLite DATETIME format). This causes `bf close` failures with the error: **"Invalid claimed_at format: premature end of input"**.

**Impact:** Beads affected by this bug become permanently unclosable via normal workflow.

---

## Expected Timestamp Format

### RFC3339 Standard (HOOP expects)

The HOOP daemon expects timestamps in **RFC3339** format, which is the internet timestamp standard defined in [RFC 3339](https://tools.ietf.org/html/rfc3339).

### Valid RFC3339 Examples

| Format | Example | Parses in HOOP? |
|--------|---------|-----------------|
| Basic RFC3339 | `2026-04-21T18:42:10Z` | ✅ Yes |
| With milliseconds | `2026-04-21T18:42:10.123Z` | ✅ Yes |
| With microseconds | `2026-04-21T18:42:10.123456Z` | ✅ Yes |
| With nanoseconds | `2026-04-21T18:42:10.123456789Z` | ✅ Yes |
| With timezone offset | `2026-04-21T18:42:10+00:00` | ✅ Yes |
| With negative offset | `2026-04-21T18:42:10-08:00` | ✅ Yes |
| With custom offset | `2026-04-21T18:42:10+05:30` | ✅ Yes |
| Space separator + T | `2026-04-21 18:42:10Z` | ✅ Yes (chrono accepts) |

### RFC3339 Format Requirements

- **Date separator:** Must be `T` (space is accepted by chrono but not standard)
- **Timezone:** Required (`Z`, `+00:00`, `-05:00`, etc.)
- **Fractional seconds:** Optional (`.1` through `.123456789`)
- **Date format:** `YYYY-MM-DD`
- **Time format:** `HH:MM:SS`

### Parsing Code Location

**File:** `hoop-daemon/src/supervisor.rs:1084-1099`

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

---

## Actual Timestamp Format Received

### SQLite DATETIME Format (from beads_rust/br CLI)

The beads_rust CLI (`br`) has code paths that produce timestamps in **SQLite's native DATETIME format**, which is **NOT RFC3339-compliant**.

### SQLite DATETIME Examples

| Format | Example | Parses in HOOP? | Why it fails |
|--------|---------|-----------------|--------------|
| **SQLite CURRENT_TIMESTAMP** | `2026-04-21 18:42:10` | ❌ **NO** | No `T` separator, no timezone |
| SQLite with milliseconds | `2026-04-21 18:42:10.123` | ❌ **NO** | Space separator, no timezone |
| Partial date | `2026-04-21` | ❌ **NO** | Missing time component |
| Empty string | `` | ❌ **NO** | No data |

### Root Cause: Divergent Code Paths in beads_rust

The beads_rust/br CLI has two code paths for writing `claimed_at`:

#### Path 1: Correct RFC3339 (beads_rust/src/storage/sqlite.rs)
```rust
// This path explicitly sets RFC3339 format
claimed_at = now.to_rfc3339()  // ✅ Correct
```

#### Path 2: SQLite DEFAULT CURRENT_TIMESTAMP (beads_rust/src/claim.rs)
```rust
// This path omits claimed_at, letting SQLite use the schema default
INSERT INTO worker_sessions (worker_id, model, harness, bead_id, workspace_path)
-- ❌ claimed_at NOT included, falls back to schema default
-- Schema: claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
-- Result: SQLite stores "2026-04-21 18:42:10" instead of "2026-04-21T18:42:10Z"
```

---

## Format Mismatches Identified

### Critical Mismatch: Space vs 'T' Separator

| Component | RFC3339 (Expected) | SQLite DATETIME (Actual) | Impact |
|-----------|-------------------|------------------------|---------|
| Date-Time separator | `T` (required) | ` ` (space) | ❌ **FAILS** |
| Timezone | Required (`Z` or `±HH:MM`) | Missing | ❌ **FAILS** |
| Date format | `YYYY-MM-DD` | `YYYY-MM-DD` | ✅ Matches |
| Time format | `HH:MM:SS` | `HH:MM:SS` | ✅ Matches |

### Comparison Table

| Timestamp String | Format | Source | Parses? | Error |
|------------------|--------|--------|---------|-------|
| `2026-04-21T18:42:10Z` | RFC3339 | ✅ Correct path | ✅ Yes | None |
| `2026-04-21T18:42:10.123Z` | RFC3339 + ms | ✅ Correct path | ✅ Yes | None |
| `2026-04-21T18:42:10+00:00` | RFC3339 + offset | ✅ Correct path | ✅ Yes | None |
| **`2026-04-21 18:42:10`** | **SQLite DATETIME** | ❌ **Path 2 bug** | ❌ **NO** | **"premature end of input"** |
| `2026-04-21` | Partial date | Data loss | ❌ NO | "premature end of input" |
| `` | Empty string | Data loss | ❌ NO | "premature end of input" |

---

## Error Messages and Failure Indicators

### Primary Error: "premature end of input"

**Full error message:**
```
Invalid claimed_at format: premature end of input
```

**When it occurs:**
- Running `bf close <bead-id>`
- Reading from `worker_sessions` table in beads_rust
- Parsing timestamp with `chrono::DateTime::parse_from_rfc3339()`

**Error code location:**
- **File:** beads_rust/br CLI (external repository)
- **Function:** Velocity calculation or timestamp parsing in `src/velocity.rs`
- **Code:**
  ```rust
  let claimed_at = DateTime::parse_from_rfc3339(&claimed_at_str)
      .map_err(|e| anyhow::anyhow!("Invalid claimed_at format: {}", e))?;  // ← Error here
      .with_timezone(&Utc);
  ```

### Secondary Error: "Invalid timestamp format"

**HOOP daemon error (when sanitizing):**
```
Invalid timestamp format '2026-04-21 18:42:10' in event: premature end of input, using current time as fallback
```

**When it occurs:**
- Processing events in HOOP daemon
- `sanitize_timestamp()` function in `hoop-daemon/src/supervisor.rs`
- HOOP falls back to current time (operation continues, but data is lost)

---

## Sample Timestamps Showing Discrepancy

### Valid Timestamps (Expected Format)

```rust
// All of these parse successfully in HOOP:
"2026-04-21T18:42:10Z"                    // Standard RFC3339
"2026-04-21T18:42:10.123Z"                // With milliseconds
"2026-04-21T18:42:10+00:00"               // With UTC offset
"2026-04-21T18:42:10-08:00"               // With negative offset (PST)
"2026-04-21T18:42:10+05:30"               // With custom offset (IST)
"2026-04-21 18:42:10Z"                    // Space separator (chrono accepts)
"2026-04-21T18:42:10.123456Z"             // Microseconds
"2026-04-21T18:42:10.123456789Z"          // Nanoseconds
```

### Invalid Timestamps (Actual Format Received)

```rust
// All of these FAIL to parse in HOOP:
"2026-04-21 18:42:10"                     // ❌ SQLite CURRENT_TIMESTAMP (PRIMARY BUG)
"2026-04-21 18:42:10.123"                 // ❌ SQLite with milliseconds
"2026-04-21"                              // ❌ Partial date only
""                                       // ❌ Empty string
"April 21, 2026"                          // ❌ Human-readable format
"not-a-timestamp"                        // ❌ Garbage data
```

### Real-World Examples from Production

**Affected beads:** `bf-2j9e`, `bf-32zd`, `bf-6mca`, `bf-5me7`, `bf-2y8s`, `bf-5i1ln`

**Example from bead `bf-5i1ln`:**
- **Timestamp stored:** `2026-04-21 18:42:10` (SQLite format)
- **Error when closing:** `Invalid claimed_at format: premature end of input`
- **Workaround:** Manual SQL DELETE of malformed rows (script: `tmp_fix_worker_sessions.py`)

---

## Validation Tests

### Test File Location
**File:** `hoop-daemon/tests/claimed_at_parsing.rs`

### Test Coverage Summary

The test suite provides comprehensive coverage of timestamp format validation:

| Test Category | Tests | Purpose |
|--------------|-------|---------|
| Valid RFC3339 formats | 9 tests | Verify correct timestamps parse |
| Invalid formats | 6 tests | Verify malformed timestamps are rejected |
| Edge cases | 13 tests | Boundary conditions, special characters |
| Integration | 2 tests | CollisionIndexEntry behavior |
| Security | 1 test | SQL injection attempts |
| Round-trip | 1 test | Timestamp preservation through storage |

### Running the Tests

```bash
# Run all claimed_at parsing tests
nix-shell --run 'cargo test --package hoop-daemon --test claimed_at_parsing'

# Expected: All 40+ tests pass
```

### Key Test Demonstrating the Bug

```rust
#[test]
fn demonstrates_premature_end_of_input_issue() {
    let invalid_timestamps = vec![
        "",                           // Empty → "premature end of input"
        "2026-04-21",                // Partial date
        "April 21, 2026",            // Wrong format
        "2026-04-21 18:42:10",      // SQLite CURRENT_TIMESTAMP format
        "not-a-timestamp",          // Garbage
    ];

    for ts in invalid_timestamps {
        let entry = create_test_entry(ts);
        
        // Entry accepts invalid timestamp (stored as TEXT in SQLite)
        assert_eq!(entry.claimed_at, ts);
        
        // But parsing fails
        let parse_result = chrono::DateTime::parse_from_rfc3339(&entry.claimed_at);
        assert!(parse_result.is_err(), "Timestamp '{}' should fail to parse", ts);
        
        // Empty string produces "premature end of input"
        if ts.is_empty() {
            let err = parse_result.unwrap_err();
            assert!(err.to_string().contains("premature end of input"));
        }
    }
}
```

---

## Data Flow Diagram

```
┌─────────────────┐
│  beads_rust/br  │
│     CLI         │
└────────┬────────┘
         │
         │ writes claimed_at
         ├──────────────────┬────────────────────┐
         │                  │                    │
         │ Path 1 (Correct) │ Path 2 (Bug)       │
         │                  │                    │
         ▼                  ▼                    ▼
    RFC3339         SQLite CURRENT       Empty/Partial
    format          TIMESTAMP format     format
  (T separator)   (space separator)   (data loss)
         │                  │                    │
         │                  │                    │
         └──────────────────┴────────────────────┘
                            │
                            │ stored in SQLite
                            ▼
                    ┌───────────────┐
                    │   worker_      │
                    │   sessions     │
                    │   table        │
                    └───────────────┘
                            │
                            │ read by bf close
                            ▼
                    ┌───────────────┐
                    │   br CLI       │
                    │   velocity.rs  │
                    └───────────────┘
                            │
                            │ DateTime::parse_from_rfc3339()
                            ▼
                    ┌───────────────┐
                    │   Result:      │
                    │                │
                    │  ✅ RFC3339    │
                    │  ❌ SQLite     │
                    └───────────────┘
```

---

## Impact Assessment

### Severity: **HIGH**

**Why HIGH:**
- **Data loss:** Affected beads become permanently unclosable
- **No recovery:** Normal workflow fails permanently
- **User impact:** Operators cannot close beads, must use manual SQL workarounds
- **Prevalence:** Multiple production beads affected

**Affected Components:**
1. **br CLI** (`bf close` command) - Primary failure point
2. **HOOP daemon** - Logs warnings, falls back to current time
3. **bead workflow** - Beads stuck in open state

**Workarounds:**
- Manual SQL DELETE of malformed `worker_sessions` rows
- Script: `tmp_fix_worker_sessions.py` (temporary)
- Risk: Data loss, manual intervention required

---

## Fix Requirements

### For beads_rust/br CLI (External Repository)

**File:** beads_rust/src/claim.rs

**Required changes:**
1. **Line 265-268:** Explicitly set `claimed_at = Utc::now().to_rfc3339()` in INSERT statement
2. **Line 354-357:** Explicitly set `claimed_at = Utc::now().to_rfc3339()` in second INSERT statement
3. **Add defensive parsing** in `src/velocity.rs:95-97` to accept both RFC3339 and SQLite DATETIME formats

**Schema change (optional):**
- Remove `DEFAULT CURRENT_TIMESTAMP` from `worker_sessions.claimed_at` column
- Make `claimed_at` required in all INSERT statements

### For HOOP Daemon

**File:** `hoop-daemon/src/supervisor.rs`

**Current behavior:** ✅ Already correct
- `sanitize_timestamp()` function handles empty/invalid timestamps
- Falls back to current time with warning
- No changes needed

**Enhancement (optional):**
- Add metrics/counters for timestamps sanitized
- Log the source of invalid timestamps for debugging

---

## Related Documentation

- **Bug report:** `docs/claimed_at_parsing_error.md`
- **Test coverage:** `docs/claimed_at_test_coverage_analysis.md`
- **Test file:** `hoop-daemon/tests/claimed_at_parsing.rs`
- **Affected beads:** `bf-2j9e`, `bf-32zd`, `bf-6mca`, `bf-5me7`, `bf-2y8s`, `bf-5i1ln`
- **Fix bead:** `bf-6af` (bead-forge repository)

---

## Recommendations

### Immediate Actions

1. **Document this analysis** - ✅ This file serves as documentation
2. **Update beads_rust/br CLI** - Fix the root cause in external repository
3. **Add regression test** - Ensure RFC3339 format is always produced
4. **Remove workaround scripts** - Once fix is deployed

### Long-term Improvements

1. **Schema validation** - Add CHECK constraint to enforce RFC3339 format in SQLite
2. **Migration script** - Convert existing SQLite DATETIME to RFC3339
3. **Monitoring** - Alert on invalid timestamp formats in production
4. **Type safety** - Use Rust types that enforce RFC3339 at compile time

---

## Conclusion

The `claimed_at` timestamp format has a **critical mismatch** between expected RFC3339 format and actual SQLite DATETIME format. This is caused by divergent code paths in the beads_rust/br CLI, where some INSERT statements omit `claimed_at` and fall back to SQLite's `DEFAULT CURRENT_TIMESTAMP`, which produces non-RFC3339 format.

**Expected:** RFC3339 format with `T` separator and timezone (`2026-04-21T18:42:10Z`)  
**Actual:** SQLite DATETIME format with space separator and no timezone (`2026-04-21 18:42:10`)  
**Error:** "Invalid claimed_at format: premature end of input"

The fix requires changes to the beads_rust/br CLI (external repository) to explicitly set RFC3339 format in all code paths. HOOP already handles this gracefully with fallback behavior, but the root cause must be fixed in the upstream dependency.
