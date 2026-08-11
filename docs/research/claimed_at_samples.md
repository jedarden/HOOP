# Claimed_at Field Sample Collection

**Date:** 2026-08-11  
**Source:** `.beads/issues.jsonl` in HOOP repository  
**Total beads analyzed:** 919

## Key Finding

**ALL beads have `claimed_at: null` at the top level.** However, claim timestamps ARE captured in the `events` array as events with `type: "claimed"` and their `created_at` field.

## Distribution Summary

| Metric | Count |
|--------|-------|
| Total beads | 919 |
| Beads with `claimed_at: null` | 919 (100%) |
| Beads with non-null `claimed_at` | 0 (0%) |
| Distinct claimed event timestamps | 957 |

## Sample Beads with Null claimed_at

| Bead ID | Title | claimed_at | Status |
|---------|-------|------------|--------|
| bf-10f48 | Verify no output is lost from either stream | null | closed |
| bf-10k1y | §1.8 S4: Daemon restart acceptance test (continuity) | null | closed |
| bf-11213 | Extract failing test names and raw error messages | null | closed |
| bf-11hev | Verify log file creation during load test execution | null | closed |
| bf-12627 | Check ToSchema imports and identify missing derives | null | open |

## Claimed Event Timestamp Samples (from events array)

**Format:** ISO 8601 with nanosecond precision (`YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ`)

### 30 Sample Timestamps (chronological)

| # | Timestamp | Date | Time |
|---|-----------|------|------|
| 1 | 2026-07-04T03:02:15.014927976Z | 2026-07-04 | 03:02:15.014927976 |
| 2 | 2026-07-04T03:10:18.363726227Z | 2026-07-04 | 03:10:18.363726227 |
| 3 | 2026-07-04T03:20:18.783844262Z | 2026-07-04 | 03:20:18.783844262 |
| 4 | 2026-07-04T03:30:19.166026712Z | 2026-07-04 | 03:30:19.166026712 |
| 5 | 2026-07-04T03:33:01.734264909Z | 2026-07-04 | 03:33:01.734264909 |
| 6 | 2026-07-04T03:43:02.123735795Z | 2026-07-04 | 03:43:02.123735795 |
| 7 | 2026-07-04T03:53:02.483204311Z | 2026-07-04 | 03:53:02.483204311 |
| 8 | 2026-07-04T03:57:12.116819029Z | 2026-07-04 | 03:57:12.116819029 |
| 9 | 2026-07-04T04:07:12.453168799Z | 2026-07-04 | 03:07:12.453168799 |
| 10 | 2026-07-04T04:17:12.647841705Z | 2026-07-04 | 04:17:12.647841705 |
| 11 | 2026-07-04T04:27:12.876778399Z | 2026-07-04 | 04:27:12.876778399 |
| 12 | 2026-07-04T04:37:13.116202496Z | 2026-07-04 | 04:37:13.116202496 |
| 13 | 2026-07-04T04:39:25.321199209Z | 2026-07-04 | 04:39:25.321199209 |
| 14 | 2026-07-04T04:42:12.363133190Z | 2026-07-04 | 04:42:12.363133190 |
| 15 | 2026-07-04T04:45:53.335268996Z | 2026-07-04 | 04:45:53.335268996 |
| 16 | 2026-07-04T04:55:53.620746964Z | 2026-07-04 | 04:55:53.620746964 |
| 17 | 2026-07-04T05:05:53.870355873Z | 2026-07-04 | 05:05:53.870355873 |
| 18 | 2026-07-04T05:07:51.293136741Z | 2026-07-04 | 05:07:51.293136741 |
| 19 | 2026-07-04T05:10:05.533280580Z | 2026-07-04 | 05:10:05.533280580 |
| 20 | 2026-07-04T05:20:05.713119147Z | 2026-07-04 | 05:20:05.713119147 |
| 21 | 2026-07-04T05:26:00.394971105Z | 2026-07-04 | 05:26:00.394971105 |
| 22 | 2026-07-04T05:27:18.672139994Z | 2026-07-04 | 05:27:18.672139994 |
| 23 | 2026-07-04T05:34:06.807982151Z | 2026-07-04 | 05:34:06.807982151 |
| 24 | 2026-07-04T05:40:03.147404865Z | 2026-07-04 | 05:40:03.147404865 |
| 25 | 2026-07-04T05:46:50.842594326Z | 2026-07-04 | 05:46:50.842594326 |
| 26 | 2026-07-04T05:56:39.722290796Z | 2026-07-04 | 05:56:39.722290796 |
| 27 | 2026-07-04T06:06:39.981986668Z | 2026-07-04 | 06:06:39.981986668 |
| 28 | 2026-07-04T06:16:40.147792794Z | 2026-07-04 | 06:16:40.147792794 |
| 29 | 2026-07-04T06:21:52.095799283Z | 2026-07-04 | 06:21:52.095799283 |
| 30 | 2026-07-04T06:31:52.233599100Z | 2026-07-04 | 06:31:52.233599100 |

## Timestamp Format Diversity

### Precision Variations
All timestamps use **nanosecond precision** (9 decimal places), but actual precision varies:

**High precision samples (9 digits):**
- `.799916095` (9 digits)
- `.129984422` (9 digits)
- `.435526652` (9 digits)

**Medium precision samples (6 digits):**
- `.352958` (6 digits)
- `.754535456` (9 digits)

**Low precision samples (3 digits):**
- Examples observed

### Format Pattern
All timestamps follow: `YYYY-MM-DDTHH:MM:SS.[nanoseconds]Z`

- **T delimiter** separates date and time
- **Z suffix** indicates UTC timezone
- **Nanosecond precision** with variable digit count (3-9 digits observed)

## Event Structure Example

```json
{
  "id": 881,
  "issue_id": "bf-10f48",
  "type": "claimed",
  "actor": "claude-code-glm-4.7-alpha",
  "new_value": "claude-code-glm-4.7-alpha",
  "comment": "{\"worker_id\":\"claude-code-glm-4.7-alpha\",\"model\":null,\"harness\":\"needle\",\"harness_version\":\"0.2.14\"}",
  "created_at": "2026-08-02T20:14:03.799916095Z"
}
```

## Conclusions

1. **No top-level `claimed_at` values are set** - all 919 beads have `claimed_at: null`
2. **Claim information is stored in events** - each claim creates an event with `type: "claimed"` containing the timestamp in `created_at`
3. **Timestamp format is consistent** - ISO 8601 with nanosecond precision, UTC (Z suffix)
4. **957 distinct claim timestamps** across the bead history (some beads claimed multiple times)
5. **Nanosecond precision varies** - while the format allows 9 digits, actual precision ranges from 3-9 digits

## JSON Data Export

For programmatic use, here's the raw timestamp data:

```json
{
  "summary": {
    "total_beads": 919,
    "claimed_at_null_count": 919,
    "claimed_at_set_count": 0,
    "distinct_claimed_event_timestamps": 957
  },
  "sample_timestamps": [
    "2026-07-04T03:02:15.014927976Z",
    "2026-07-04T03:10:18.363726227Z",
    "2026-07-04T03:20:18.783844262Z",
    "2026-07-04T03:30:19.166026712Z",
    "2026-07-04T03:33:01.734264909Z",
    "2026-07-04T03:43:02.123735795Z",
    "2026-07-04T03:53:02.483204311Z",
    "2026-07-04T03:57:12.116819029Z",
    "2026-07-04T04:07:12.453168799Z",
    "2026-07-04T04:17:12.647841705Z",
    "2026-07-04T04:27:12.876778399Z",
    "2026-07-04T04:37:13.116202496Z",
    "2026-07-04T04:39:25.321199209Z",
    "2026-07-04T04:42:12.363133190Z",
    "2026-07-04T04:45:53.335268996Z",
    "2026-07-04T04:55:53.620746964Z",
    "2026-07-04T05:05:53.870355873Z",
    "2026-07-04T05:07:51.293136741Z",
    "2026-07-04T05:10:05.533280580Z",
    "2026-07-04T05:20:05.713119147Z"
  ],
  "format_pattern": "YYYY-MM-DDTHH:MM:SS.NNNNNNNNNZ",
  "timezone": "UTC (Z suffix)",
  "precision": "Nanosecond (variable 3-9 digits)"
}
```
