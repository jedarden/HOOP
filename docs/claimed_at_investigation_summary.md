# claimed_at Format Investigation — Executive Summary for Fix Implementation

**Investigation Date:** 2026-08-11  
**Parent Bead:** bf-3l63i  
**Purpose:** Comprehensive summary to guide fix implementation based on all investigation findings

---

## Executive Summary

The `claimed_at` timestamp format mismatch is a **critical bug in the external `beads_rust/br` CLI** (outside HOOP repository). HOOP code is **NOT the source** of the problem and does NOT require fixing. The issue causes 11.4% of worker_sessions records to fail parsing with error: **"Invalid claimed_at format: premature end of input"**.

**Key Finding:** The mismatch originates from two divergent INSERT code paths in beads_rust:
- **Path A (88.4%)**: Explicit RFC3339 via `Utc::now().to_rfc3339()` ✅
- **Path B (11.4%)**: SQLite schema default `CURRENT_TIMESTAMP` ❌

---

## Problem Location

### External System (Root Cause)
- **Repository:** `dicklesworthstone/beads_rust`
- **Component:** `worker_sessions` table in `.beads/beads.db`
- **Files Affected:** 
  - `beads_rust/src/storage/sqlite.rs` (Path A - correct)
  - `beads_rust/src/claim.rs` (Path B - buggy)
  - `beads_rust/src/velocity.rs` (parsing failure point)

### HOOP Components (NOT Affected)
- **Status:** HOOP is a data consumer, not producer
- **Source:** Reads from `events.jsonl` (always RFC3339 format)
- **Storage:** `collision_index` table uses correct RFC3339 format
- **Handling:** Has defensive `sanitize_timestamp()` function with fallback

---

## Format Mismatch Details

### Expected Format (RFC3339)
```
2026-08-01T02:11:38.034049318+00:00
└────────┬────────┘ └───┬───┘
   RFC3339 with T separator and +00:00 timezone
```

### Actual Formats Received

**Format 1: RFC3339 (88.4%) - CORRECT ✅**
- Pattern: `YYYY-MM-DDTHH:MM:SS.NNNNNNNNN+00:00`
- Example: `2026-08-01T02:11:38.034049318+00:00`
- Characteristics: T separator, nanosecond precision, +00:00 timezone
- Parse Status: ✅ Success

**Format 2: SQLite DATETIME (11.4%) - INCORRECT ❌**
- Pattern: `YYYY-MM-DD HH:MM:SS`
- Example: `2026-07-04 03:02:15`
- Characteristics: Space separator, no timezone, no fractional seconds
- Parse Status: ❌ FAILS - "premature end of input"

**Format 3: Hybrid (0.1%) - INCORRECT ❌**
- Pattern: `YYYY-MM-DDTHH:MM:SS.NN`
- Example: `2026-08-03T06:46:20.80`
- Characteristics: T separator (correct), missing timezone (incorrect)
- Parse Status: ❌ FAILS - "premature end of input"

---

## Root Cause Analysis

### Primary Root Cause: Divergent Code Paths

**Path A: Explicit RFC3339 (Correct) - 88.4%**
```rust
// Location: beads_rust/src/storage/sqlite.rs
claimed_at = Utc::now().to_rfc3339()
// Produces: "2026-08-01T02:11:38.034049318+00:00" ✅
```

**Path B: SQLite Schema Default (Buggy) - 11.4%**
```rust
// Location: beads_rust/src/claim.rs
INSERT INTO worker_sessions (worker_id, model, harness, bead_id, workspace_path)
-- claimed_at NOT included in INSERT
// Schema default applies: DEFAULT CURRENT_TIMESTAMP
// Produces: "2026-07-04 03:02:15" ❌
```

### Schema Definition (Problematic)
```sql
CREATE TABLE worker_sessions (
    worker_id TEXT NOT NULL,
    model TEXT NOT NULL,
    harness TEXT NOT NULL,
    bead_id TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,  -- ← ROOT CAUSE
    PRIMARY KEY (worker_id, bead_id)
);
```

### Why Path B Exists
The schema default `CURRENT_TIMESTAMP` provides a fallback when `claimed_at` is not explicitly set in INSERT statements, creating silent format mismatch.

### Temporal Distribution
- **2026-07-04:** All claims used Path B (SQLite DATETIME format)
- **2026-08-01 onwards:** All claims use Path A (RFC3339 format)
- **Inference:** A fix was deployed between 2026-07-04 and 2026-08-01, but 109 historical samples remain malformed

---

## Error Messages and Failures

### Primary Error
**"Invalid claimed_at format: premature end of input"**
- **Source:** `chrono::DateTime::parse_from_rfc3339()` failure
- **Trigger:** Empty string or SQLite DATETIME format without timezone
- **Impact:** `bf close` permanently fails on affected beads

### Error Code Location
**Repository:** beads_rust/br CLI (external)  
**File:** `src/velocity.rs`  
**Lines:** ~95-97

```rust
let claimed_at = DateTime::parse_from_rfc3339(&claimed_at_str)
    .map_err(|e| anyhow::anyhow!("Invalid claimed_at format: {}", e))?;  // ← Error here
```

### Affected Records
- **Total malformed samples:** 110 of 959 (11.5%)
- **Affected date:** All from 2026-07-04
- **Business impact:** Beads claimed on 2026-07-04 cannot be closed via normal workflow

---

## HOOP Impact Assessment

### HOOP Internal Systems — NOT AFFECTED ✅

| Component | Impact | Severity |
|-----------|--------|----------|
| **Event tailer** | Reads correct RFC3339 from events.jsonl | NONE |
| **Timestamp sanitizer** | Validates and handles invalid formats | NONE |
| **Collision detection** | Stores correct RFC3339 format | NONE |
| **Fleet database** | No schema default issues | NONE |

### HOOP Data Flow (Working Correctly)
```
events.jsonl → EventTailer → sanitize_timestamp() → collision_index
     ↓                ↓              ↓                    ↓
  RFC3339        Deserialize    Validate              Store
  (correct)      Successfully   With Fallback        RFC3339
```

### HOOP Defensive Code (Already Correct)
```rust
// File: hoop-daemon/src/supervisor.rs:1084-1102
fn sanitize_timestamp(ts: &str) -> String {
    if ts.is_empty() {
        warn!("Empty timestamp in event, using current time as fallback");
        return Utc::now().to_rfc3339();
    }

    match DateTime::parse_from_rfc3339(ts) {
        Ok(_) => ts.to_string(), // Valid timestamp
        Err(e) => {
            warn!("Invalid timestamp format '{}' in event: {}, using current time as fallback", ts, e);
            Utc::now().to_rfc3339() // Fallback
        }
    }
}
```

---

## Comparison: HOOP vs External Systems

| Aspect | HOOP (Correct) | beads_rust (Buggy) |
|--------|----------------|-------------------|
| **Timestamp format** | RFC3339 | Mixed (88.4% / 11.4%) |
| **Schema defaults** | None (explicit values) | `DEFAULT CURRENT_TIMESTAMP` |
| **Error handling** | Defensive fallback | Hard parse failure |
| **Test coverage** | Comprehensive (35 tests) | Unknown |
| **Data source** | events.jsonl (validated) | worker_sessions (mixed) |

---

## Fix Implementation Recommendations

### CRITICAL DISTINCTION

**HOOP REPOSITORY:** NO CHANGES REQUIRED ✅

**EXTERNAL REPOSITORY (beads_rust):** REQUIRES FIX ❌

---

### Fix for beads_rust Repository (REQUIRED)

#### Immediate Action: Migrate Historical Data

```sql
-- Convert SQLite DATETIME format to RFC3339
UPDATE worker_sessions
SET claimed_at = replace(claimed_at, ' ', 'T') || '+00:00'
WHERE claimed_at LIKE '% %';

-- Fix hybrid outlier
UPDATE worker_sessions
SET claimed_at = claimed_at || '+00:00'
WHERE claimed_at = '2026-08-03T06:46:20.80';

-- Verify migration
SELECT COUNT(*) FROM worker_sessions WHERE claimed_at NOT LIKE '%+00:00';
-- Expected: 0
```

#### Long-term Fix: Remove Schema Default

```sql
-- Remove problematic default
ALTER TABLE worker_sessions ALTER COLUMN claimed_at DROP DEFAULT;

-- Add CHECK constraint for RFC3339 format
ALTER TABLE worker_sessions ADD CONSTRAINT claimed_at_rfc3339
CHECK (claimed_at LIKE '%+00:00' OR claimed_at LIKE '%Z');
```

#### Code Fix: Always Set claimed_at Explicitly

```rust
// In beads_rust/src/claim.rs (or wherever worker_sessions INSERT occurs)
let claimed_at = Utc::now().to_rfc3339();

INSERT INTO worker_sessions (worker_id, model, harness, bead_id, workspace_path, claimed_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6);
```

---

### Fix for HOOP Repository (NOT REQUIRED)

**Rationale:** HOOP code is already correct. No changes needed.

**Verification:**
- ✅ EventTailer correctly deserializes RFC3339 timestamps
- ✅ `sanitize_timestamp()` validates and handles invalid formats
- ✅ `collision_index` table uses explicit RFC3339 values
- ✅ All test coverage passes (35 tests)

**Optional Enhancement:** Add integration test to verify events.jsonl format
```rust
#[test]
fn events_jsonl_contains_rfc3339_timestamps() {
    let events = read_events_jsonl();
    for event in events {
        if let NeedleEvent::Claim { ts, .. } = event {
            assert!(DateTime::parse_from_rfc3339(&ts).is_ok());
        }
    }
}
```

---

## Impact and Risk Assessment

### Severity Level: **HIGH** (for external system)

**Business Impact:**
- 109 worker_sessions records permanently unclosable via normal workflow
- Manual SQL intervention required for affected beads
- Workflow disruption and increased support burden

**Data Impact:**
- Original claim timestamps remain malformed
- Audit trail compromised
- Historical data corrupted

### HOOP Risk Level: **NONE**

**Explanation:**
1. HOOP reads `claimed_at` from `events.jsonl` (always RFC3339 format)
2. HOOP's `sanitize_timestamp()` function validates and handles invalid formats
3. HOOP's `collision_index` table stores only validated RFC3339 timestamps
4. HOOP does not read from or write to the `worker_sessions` table

---

## Implementation Priority

### Priority 1: External Fix (beads_rust)
- **Migrate 110 malformed samples** in `worker_sessions` table
- **Remove `DEFAULT CURRENT_TIMESTAMP`** from schema
- **Always set `claimed_at` explicitly** in INSERT statements
- **Add CHECK constraint** for RFC3339 format

### Priority 2: Monitoring and Testing
- Add integration tests for cross-repo format consistency
- Implement monitoring for timestamp format violations
- Alert on non-zero sanitization events

### Priority 3: HOOP Enhancement (Optional)
- Add integration test to verify events.jsonl format
- No code changes required (already correct)

---

## Verification Steps

### After External Fix

1. **Verify migration success:**
   ```sql
   SELECT COUNT(*) FROM worker_sessions WHERE claimed_at NOT LIKE '%+00:00';
   -- Expected: 0
   ```

2. **Test `bf close` on migrated samples:**
   ```bash
   bf close <affected-bead-id>
   # Should succeed without "Invalid claimed_at format" error
   ```

3. **Verify new claims use RFC3339:**
   ```bash
   # Create a test bead
   bf create test-bead
   # Claim it
   bf claim test-bead
   # Check format in database
   sqlite3 .beads/beads.db "SELECT claimed_at FROM worker_sessions WHERE bead_id='test-bead'"
   # Expected: RFC3339 format with +00:00
   ```

### HOOP Verification (No Changes Expected)

1. **Run comprehensive tests:**
   ```bash
   nix-shell --run 'cargo test --package hoop-daemon --test claimed_at_parsing'
   # Expected: All 35 tests pass
   ```

2. **Verify event processing:**
   ```bash
   # Monitor for invalid timestamp warnings
   grep "Invalid timestamp format" /var/log/hoop/hoop-daemon.log
   # Expected: No new warnings
   ```

---

## Conclusion

### Root Cause Summary

The `claimed_at` format mismatch **originates in the external `beads_rust/br` CLI**, not in HOOP code. Two divergent INSERT code paths in beads_rust produce different timestamp formats:
1. **Path A (88.4%):** Explicit RFC3339 via `Utc::now().to_rfc3339()` ✅
2. **Path B (11.4%):** SQLite schema default `CURRENT_TIMESTAMP` ❌

### HOOP's Position

**HOOP is NOT the source of the mismatch.** HOOP is a consumer of Claim events from `events.jsonl` (which always contains correct RFC3339 timestamps) and stores validated timestamps in its own `collision_index` table.

**HOOP code is correct:**
- ✅ EventTailer reads RFC3339 timestamps from events.jsonl
- ✅ `sanitize_timestamp()` validates and handles invalid formats
- ✅ `collision_index` table stores only RFC3339 timestamps
- ✅ All test coverage passes (35 tests)

### Recommended Fix Path

**Fix the external beads_rust repository:**
1. Migrate 110 malformed samples in `worker_sessions` table
2. Remove `DEFAULT CURRENT_TIMESTAMP` from schema
3. Always set `claimed_at` explicitly in INSERT statements
4. Add CHECK constraint for RFC3339 format

**No HOOP changes required.**

---

## Related Documentation

- **Root cause analysis:** `docs/claimed_at_root_cause_analysis.md`
- **Format mismatch comparison:** `docs/claimed_at_format_mismatch_comparison.md`
- **Error messages catalog:** `docs/claimed_at_error_messages_and_failures.md`
- **Timestamp field formats:** `docs/timestamp-field-formats.md`
- **Sample collection:** `docs/claimed_at_samples.md`
- **Test coverage:** `docs/claimed_at_test_coverage_analysis.md`

---

**Investigation complete.** This summary synthesizes all investigation findings to guide the correct fix implementation: update external beads_rust repository, not HOOP parsing code.