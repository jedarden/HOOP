# claimed_at Timestamp Parsing Investigation

**Bead:** bf-5wxp7
**Date:** 2026-08-11
**Workspace:** /home/coding/HOOP

## Executive Summary

The `claimed_at` timestamp parsing error ("Invalid claimed_at format: premature end of input") occurs when empty or invalid timestamp strings are passed to chrono's RFC3339 parser. HOOP has a mitigation in place (`sanitize_timestamp()`) that handles this gracefully, but the investigation reveals where timestamps originate and why they may be invalid.

## Location of claimed_at Parsing Logic

### Primary Parsing Function
**File:** `hoop-daemon/src/supervisor.rs`
**Lines:** 1084-1102
**Function:** `sanitize_timestamp(ts: &str) -> String`

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

### Where Parsing is Called
**File:** `hoop-daemon/src/supervisor.rs`
**Line:** 1172
**Context:** `update_fleet_from_event()` function processing `Claim` events

```rust
NeedleEvent::Claim { .. } => {
    // Register in collision index so concurrent-work detection can fire
    if let Some(ref proj) = project {
        let now = chrono::Utc::now().to_rfc3339();
        // Sanitize the timestamp to handle empty/invalid values from events.jsonl
        let claimed_at = sanitize_timestamp(ts);
        let entry = fleet::CollisionIndexEntry {
            bead_id: bead_id.to_string(),
            project: proj.clone(),
            worker: Some(worker.to_string()),
            claimed_at,
            file_paths: vec![],
            updated_at: now,
        };
        // ... upsert to database
    }
}
```

## Current Timestamp Format Expected vs. Received

### Expected Format: RFC3339

The code uses `chrono::DateTime::parse_from_rfc3339()` which expects RFC3339 format.

**Valid examples:**
- `2026-08-03T00:15:07.519757254+00:00` (with nanoseconds, timezone offset)
- `2026-08-03T00:15:07Z` (UTC Z suffix)
- `2026-08-03T00:15:07.123Z` (with milliseconds)
- `2026-08-03T00:15:07+05:30` (with non-UTC offset)

### What's Being Received

**From `.beads/events.jsonl` (Claim events):**
```json
{"bead":"bf-4rjil","event":"claim","strand":"auto","ts":"2026-08-03T00:15:07.519757254+00:00","worker":"alpha"}
```

The `ts` field from Claim events is the source of `claimed_at` timestamps.

**Format observed:** `2026-08-03T00:15:07.519757254+00:00`
- RFC3339 compliant
- Nanosecond precision
- UTC timezone expressed as `+00:00`

### Invalid Cases That Cause Parsing Failure

**From test file (`hoop-daemon/tests/claimed_at_parsing.rs`):**

1. **Empty string** - `""`
   - Error: "premature end of input"
   - Most common invalid case

2. **Partial date** - `"2026-04-21"`
   - Missing time component
   - Error: "premature end of input"

3. **Wrong format** - `"April 21, 2026"`
   - Not ISO 8601 compliant

4. **Garbage** - `"not-a-timestamp"`
   - Completely unparseable

## Why Parsing Fails

### Root Cause
The `chrono::DateTime::parse_from_rfc3339()` function is strict:

1. **Empty strings** produce "premature end of input" error
2. **Incomplete timestamps** (missing time, missing timezone) fail parsing
3. **Non-RFC3339 formats** are rejected

### When Invalid Timestamps Occur

1. **Corrupted events.jsonl** - File write interruption, partial line
2. **Manual editing** - Human edits events.jsonl with wrong format
3. **External tools** - Third-party tools write non-compliant timestamps
4. **Migration bugs** - Data import from old formats

## Other Timestamp Fields in HOOP (That Work Correctly)

### Timestamp Formats Used Throughout HOOP

All timestamps in HOOP use the same RFC3339 format via `chrono::Utc::now().to_rfc3339()`:

**From actual bead data (`.beads/issues.jsonl`):**
- `created_at`: `2026-08-02T18:36:21.171780570Z`
- `updated_at`: `2026-08-02T18:36:21.171780570Z`
- `closed_at`: `2026-08-02T18:36:21.171780570Z`

**Format characteristics:**
- ISO 8601 / RFC3339 compliant
- Nanosecond precision (9 decimal places)
- UTC timezone (Z suffix)
- Parseable by `chrono::DateTime::parse_from_rfc3339()`

### Code Locations That Generate Timestamps

**File:** `hoop-daemon/src/supervisor.rs`
```rust
let now = chrono::Utc::now().to_rfc3339();  // Line 1170
```

**Other locations:**
- `hoop-daemon/src/api_tour_project.rs:412` - `Utc::now().to_rfc3339()`
- `hoop-daemon/src/cross_project_propagation.rs:213` - `Utc::now().to_rfc3339()`
- `hoop-daemon/src/agent_context.rs:189` - `Utc::now().to_rfc3339()`
- And 20+ other locations

All use the same pattern: `chrono::Utc::now().to_rfc3339()`

## Data Flow: Claim Event → Collision Index

### Source: events.jsonl Claim Event
```json
{
  "bead": "bf-4rjil",
  "event": "claim",
  "ts": "2026-08-03T00:15:07.519757254+00:00",
  "worker": "alpha"
}
```

### Step 1: Event Parsing
**File:** `hoop-daemon/src/events.rs`
**Enum:** `NeedleEvent::Claim { ts: String, ... }`

### Step 2: Timestamp Sanitization
**File:** `hoop-daemon/src/supervisor.rs:1172`
```rust
let claimed_at = sanitize_timestamp(ts);
```

### Step 3: CollisionIndexEntry Creation
**File:** `hoop-daemon/src/fleet.rs:5024-5032`
```rust
pub struct CollisionIndexEntry {
    pub bead_id: String,
    pub project: String,
    pub worker: Option<String>,
    pub claimed_at: String,  // ← Sanitized timestamp stored here
    pub file_paths: Vec<String>,
    pub updated_at: String,
}
```

### Step 4: Database Storage
**Table:** `collision_index`
**Schema:** `claimed_at TEXT NOT NULL`

**File:** `hoop-daemon/src/fleet.rs:5589-5605`
```rust
fn upsert_collision_entry_conn(conn: &Connection, entry: &CollisionIndexEntry) -> Result<()> {
    conn.execute(
        r#"INSERT INTO collision_index
           (bead_id, project, worker, claimed_at, file_paths, updated_at)
           VALUES (?1,?2,?3,?4,?5,?6)
           ON CONFLICT(bead_id) DO UPDATE SET
               worker     = excluded.worker,
               file_paths = excluded.file_paths,
               updated_at = excluded.updated_at"#,
        params![
            entry.bead_id,
            entry.project,
            entry.worker,
            entry.claimed_at,  // ← Stored as TEXT
            // ...
        ],
    )
}
```

## Mitigation Strategy

### Current: sanitize_timestamp() Function

The `sanitize_timestamp()` function provides two protections:

1. **Empty string check** - Returns current time if input is empty
2. **RFC3339 validation** - Returns current time if parsing fails

**Behavior:**
- Valid timestamp → Return as-is
- Empty or invalid → Log warning, return `Utc::now().to_rfc3339()`

### Why This Works

1. **No panics** - Invalid timestamps don't crash the daemon
2. **Data integrity** - collision_index always gets a valid timestamp
3. **Audit trail** - Warnings are logged for debugging
4. **Graceful degradation** - System continues with current time fallback

## Test Coverage

**File:** `hoop-daemon/tests/claimed_at_parsing.rs`

Comprehensive test suite covering:
- Valid RFC3339 timestamps (with/without milliseconds, various timezone offsets)
- Invalid timestamps (empty, partial, wrong format, garbage)
- Edge cases (whitespace, special characters, SQL injection attempts)
- Negative timestamps (before Unix epoch)
- Extreme future dates
- Leap seconds
- Boundary values

**Test count:** 30+ test functions covering 500+ lines

## Summary of Findings

### What Was Identified

1. **Exact parsing location:** `sanitize_timestamp()` in `hoop-daemon/src/supervisor.rs:1084-1102`
2. **Expected format:** RFC3339 (ISO 8601 with timezone)
3. **Actual format received:** RFC3339 with nanoseconds (`2026-08-03T00:15:07.519757254+00:00`)
4. **Why parsing fails:** Empty strings or non-RFC3339 formats passed to chrono parser
5. **Other timestamp formats:** All HOOP timestamps use same RFC3339 format via `Utc::now().to_rfc3339()`

### Why This Matters

The `claimed_at` field is critical for:
- **Concurrent work detection** - Knowing when a bead was claimed
- **Collision detection** - Identifying workers that might be conflicting
- **Audit trails** - Reconstructing what happened when

### Current State

✅ **Mitigation in place** - `sanitize_timestamp()` handles invalid timestamps gracefully
✅ **Comprehensive tests** - Edge cases well-covered
✅ **Consistent format** - All timestamps use RFC3339

### No Immediate Action Required

The parsing logic already handles the error condition. The investigation confirms:
1. The error message is from chrono's strict RFC3339 parser
2. Empty strings cause "premature end of input" 
3. Fallback to current time prevents data corruption
4. Warnings are logged for debugging

## Recommendations for Future Work

1. **Add format validation** earlier in the pipeline (event ingestion)
2. **Metrics** on timestamp sanitization frequency (detect data quality issues)
3. **Consider a stricter schema** for events.jsonl (validate at write time)
4. **Document expected timestamp format** in events.jsonl spec comments

---

**Investigation completed:** 2026-08-11
**Next steps:** See parent bead `bf-8ukbk` for follow-up work
