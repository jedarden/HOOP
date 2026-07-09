# claimed_at Parsing Error — Minimal Reproduction & Documentation

## Summary

**Error:** `Invalid claimed_at format: premature end of input`

**Impact:** `bf close` permanently fails on beads affected by this bug, preventing normal bead closure workflow.

**Root Cause:** SQLite's `CURRENT_TIMESTAMP` default produces non-RFC3339 format.

## Exact Input Format That Fails

The error occurs when `claimed_at` is in SQLite's native DATETIME format instead of RFC3339:

| Format | Example | Parses? |
|--------|---------|---------|
| RFC3339 (correct) | `2026-04-21T18:42:10Z` | ✅ Yes |
| RFC3339 with offset | `2026-04-21T18:42:10+00:00` | ✅ Yes |
| RFC3339 with milliseconds | `2026-04-21T18:42:10.123Z` | ✅ Yes |
| **SQLite CURRENT_TIMESTAMP** | **`2026-04-21 18:42:10`** | ❌ **NO** |
| Partial (date only) | `2026-04-21` | ❌ No |
| Empty string | `` | ❌ No |

## Root Cause Analysis

### Divergent Code Paths

**Path 1: Correct RFC3339 (src/storage/sqlite.rs:1487)**
```rust
record_worker_session() explicitly sets:
claimed_at = now.to_rfc3339()  // ✅ Correct
```

**Path 2: SQLite CURRENT_TIMESTAMP (src/claim.rs:265-268, 354-357)**
```rust
INSERT INTO worker_sessions (worker_id, model, harness, bead_id, workspace_path)
-- ❌ claimed_at NOT included, falls back to schema default
-- Schema: claimed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
```

### The Failure Point (src/velocity.rs:95-97)

```rust
let claimed_at = DateTime::parse_from_rfc3339(&claimed_at_str)
    .map_err(|e| anyhow::anyhow!("Invalid claimed_at format: {}", e))?  // ← Error here
    .with_timezone(&Utc);
```

When `bf close` runs, it:
1. Queries worker_sessions for the bead
2. Finds a row with `claimed_at = "2026-04-21 18:42:10"` (SQLite format)
3. Attempts to parse as RFC3339
4. **Fails:** "premature end of input" because `parse_from_rfc3339` expects `T` separator and timezone

## Minimal Reproduction Case

The test suite at `hoop-daemon/tests/claimed_at_parsing.rs` demonstrates the issue:

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

## Expected vs Actual Behavior

### Expected
1. All `worker_sessions` rows store `claimed_at` in RFC3339 format
2. `bf close` successfully parses any `claimed_at` value
3. If a malformed timestamp exists, it's handled gracefully (skip/warn, not abort)

### Actual
1. Some `worker_sessions` rows have SQLite's native DATETIME format
2. `bf close` fails on affected beads with "Invalid claimed_at format: premature end of input"
3. Beads become permanently unclosable via normal workflow

## Evidence from Production

Multiple beads have been affected:
- `bf-2j9e` (documented in notes/bf-2j9e.md)
- `bf-32zd`, `bf-6mca`, `bf-5me7`, `bf-2y8s` (traces in bead-forge/.beads/traces/)

Workaround script `tmp_fix_worker_sessions.py` was created to manually DELETE malformed rows.

## Fix Required

See bead `bf-6af` in bead-forge for complete fix requirements:

1. **src/claim.rs:** Both INSERT statements must explicitly set `claimed_at = Utc::now().to_rfc3339()`
2. **src/velocity.rs:** Defensive parsing — accept both RFC3339 and SQLite DATETIME, or skip malformed rows instead of erroring
3. Add regression test for non-RFC3339 claimed_at
4. Remove workaround scripts once fixed

## Test Execution

```bash
# Run the reproduction test
nix-shell --run 'cargo test --package hoop-daemon --test claimed_at_parsing'

# Expected: All 12 tests pass
```

## Related Beads

- **bf-2qo1u** (HOOP): Original report of `bf close` failure
- **bf-4jylx** (HOOP): This documentation and reproduction task
- **bf-6af** (bead-forge): Root cause fix in br CLI
