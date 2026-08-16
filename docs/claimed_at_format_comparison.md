# `claimed_at` format comparison

**Task:** `hoop-36980b58`
**Purpose:** Show the timestamp contract beside the formats found in the collected samples.

## Short answer

`claimed_at` is expected to be an RFC3339 timestamp. The expected shape is:

```text
YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)
```

Examples include `2026-04-21T18:42:10Z` and
`2026-08-01T02:11:38.034049318+00:00`.

The collected `worker_sessions` samples contain two important mismatches:

1. `2026-07-04 03:02:15` — SQLite `CURRENT_TIMESTAMP` style: a space replaces
   the RFC3339 `T`, and the timezone is absent.
2. `2026-08-03T06:46:20.80` — the separator and fractional seconds are present,
   but the required timezone is absent.

Both malformed forms fail `chrono::DateTime::parse_from_rfc3339()` because they
do not contain a timezone. A Unix epoch value was not found in the collected
`claimed_at` samples; do not confuse Unix seconds used by other fields with this
field’s RFC3339 contract.

## Expected versus actual

| Sample/source | Expected RFC3339 | Actual value | Mismatch type | Parser/storage result |
|---|---|---|---|---|
| Test: basic UTC | `YYYY-MM-DDTHH:MM:SSZ` | `2026-04-21T18:42:10Z` | None | Parses successfully |
| Test: explicit offset | `YYYY-MM-DDTHH:MM:SS±HH:MM` | `2026-04-21T18:42:10+00:00` | None | Parses successfully |
| Production NEEDLE event | RFC3339 with optional fraction and timezone | `2026-08-03T00:15:07.519757254+00:00` | None; nanosecond precision is valid | HOOP preserves it |
| `worker_sessions` sample | RFC3339 requires `T` and `Z`/offset | `2026-07-04 03:02:15` | **Missing timezone**; also **space separator** under the RFC3339 contract | Fails strict RFC3339 parsing with `premature end of input` |
| `worker_sessions` sample | RFC3339 requires `T` and `Z`/offset | `2026-08-03T06:46:20.80` | **Missing timezone** | Fails strict RFC3339 parsing with `premature end of input` |
| Parser test edge case | Full date and time with timezone | `2026-04-21` | **Date only**; time and timezone missing | Rejected; HOOP falls back to `Utc::now()` |
| Parser test edge case | RFC3339 | `April 21, 2026` | **Human-readable text** | Rejected; HOOP falls back to `Utc::now()` |
| Parser test edge case | RFC3339 | `not-a-timestamp` | **Garbage/non-timestamp** | Rejected; HOOP falls back to `Utc::now()` |

### Component-level view

| Component | Expected | Observed valid sample | Observed mismatch | Effect |
|---|---|---|---|---|
| Date | `YYYY-MM-DD` | `2026-08-01` | Same | No mismatch |
| Date/time separator | `T` in RFC3339 | `T` | Space: `2026-07-04 03:02:15` | Violates the format contract; the space alone is not the whole parse failure because Chrono is lenient about a space when a timezone is present |
| Clock time | `HH:MM:SS` | `02:11:38` | `03:02:15` | No mismatch |
| Fractional seconds | Optional, 1–9 digits | `.034049318` | None in SQLite sample; `.80` in hybrid sample | Missing precision is allowed; it is not independently invalid |
| Timezone | Required: `Z` or `±HH:MM` | `+00:00` | Missing in both malformed examples | **Primary parse failure** |
| Representation | Text RFC3339 | `2026-08-01T02:11:38.034049318+00:00` | Unix epoch was not observed | A numeric epoch would be a separate **wrong representation** mismatch |

## Concrete sample groups

### 1. RFC3339 samples — expected and valid

The production event sample collection reports NEEDLE claim timestamps with
nanosecond precision and an explicit UTC offset:

```text
2026-08-03T00:15:07.519757254+00:00
2026-08-03T00:25:08.041828949+00:00
2026-08-03T00:32:48.337162852+00:00
2026-08-03T00:48:34.254529768+00:00
2026-08-03T01:21:10.591679541+00:00
```

These are valid RFC3339 values. The number of fractional digits is flexible;
the nine-digit production values are valid, not a separate required format.

### 2. SQLite DATETIME samples — two mismatches in one value

```text
Expected: RFC3339, for example 2026-08-01T02:11:38.034049318+00:00
Actual:   2026-07-04 03:02:15
          ^          ^
          |          missing timezone (`Z` or `±HH:MM`)
          space replaces the RFC3339 `T`
```

Mismatch labels:

- **Space separator:** the value is SQLite’s `YYYY-MM-DD HH:MM:SS` form rather
  than the RFC3339 `T` form.
- **Missing timezone:** the value has no `Z` or numeric offset, so it cannot be
  interpreted as an RFC3339 instant.
- **No fractional seconds:** not itself a mismatch; RFC3339 fractions are
  optional.

The format-mismatch sample report attributes this form to a
`worker_sessions.claimed_at` schema default of `CURRENT_TIMESTAMP`. That is an
external beads-rust/br data path, not the HOOP `collision_index` write path.

### 3. Hybrid sample — one mismatch

```text
Expected: RFC3339, for example 2026-08-01T02:11:38.034049318+00:00
Actual:   2026-08-03T06:46:20.80
          ----------------------^
          missing timezone (`Z` or `±HH:MM`)
```

The `T` separator and two fractional digits are acceptable components. The
timezone is mandatory, so the complete value is not RFC3339 and parsing fails.

### 4. Synthetic parser edge cases

The test suite intentionally exercises values that are not reported as
production samples:

| Value | Label | Why it fails |
|---|---|---|
| `""` | Empty | No date, time, or timezone; reproduces `premature end of input` |
| `2026-04-21` | Partial date | Time and timezone are missing |
| `April 21, 2026` | Wrong format | Human-readable text is not RFC3339 |
| `not-a-timestamp` | Garbage | No timestamp components are present |
| `2026-04-21T18:42:10` | Missing timezone | Date and time are present, but no `Z` or offset |

These examples explain the failure mode and test coverage; they should not be
counted as additional production mismatches.

## Reported sample counts

The existing format-mismatch analysis reports 959 `worker_sessions` values:

| Reported category | Count | Reported share | Format assessment |
|---|---:|---:|---|
| RFC3339 with explicit offset | 848 | 88.4% | Expected; parses |
| SQLite DATETIME with space and no timezone | 109 | 11.4% | Mismatch; does not parse |
| Hybrid `T` form without timezone | 1 | 0.1% | Mismatch; does not parse |

The listed subtotals add to 958, not 959. Treat the percentages and category
coverage as the prior report’s figures until the underlying query is rerun;
the discrepancy does not change the format mismatches or their examples.

## Which code path sees which value?

There are two paths that should not be conflated:

```text
NEEDLE events.jsonl (`event: claim`, `ts: RFC3339`)
        │
        ▼
HOOP `NeedleEvent::Claim { ts, ... }`
        │
        ▼
`sanitize_timestamp(ts)` in supervisor.rs
        │  valid → preserve input
        │  invalid/empty → log warning and use `Utc::now().to_rfc3339()`
        ▼
HOOP `collision_index.claimed_at` (TEXT)

External beads-rust/br `worker_sessions.claimed_at`
        │
        ├─ explicit `Utc::now().to_rfc3339()` path → RFC3339
        └─ SQLite `CURRENT_TIMESTAMP` default → `YYYY-MM-DD HH:MM:SS`
```

HOOP’s sanitizer is therefore a validation/fallback boundary for incoming
event timestamps. The 109 reported SQLite-style historical values belong to the
external `worker_sessions` table described in the root-cause analysis; they are
not evidence that HOOP’s `collision_index` writer emits SQLite DATETIME values.

## Code and sample references

All paths below are relative to the repository root.

- **Parser and fallback:** [`hoop-daemon/src/supervisor.rs`](../hoop-daemon/src/supervisor.rs), `sanitize_timestamp()` at lines 1094–1112.
- **Claim call site:** [`hoop-daemon/src/supervisor.rs`](../hoop-daemon/src/supervisor.rs), `update_fleet_from_event()` at lines 1177–1187.
- **HOOP storage type:** [`hoop-daemon/src/fleet.rs`](../hoop-daemon/src/fleet.rs), `CollisionIndexEntry` at lines 7705–7714; `collision_index.claimed_at` is `TEXT NOT NULL` at lines 4970–4979.
- **Event shape:** [`hoop-daemon/src/events.rs`](../hoop-daemon/src/events.rs), `NeedleEvent::Claim` carries `ts: String`.
- **Parser tests and concrete edge cases:** [`hoop-daemon/tests/claimed_at_parsing.rs`](../hoop-daemon/tests/claimed_at_parsing.rs), constants at lines 20–44 and edge-case matrix at lines 204–217.
- **Production/event samples:** [`docs/claimed_at_samples.md`](claimed_at_samples.md), “Production NEEDLE Events” and “Test Cases” sections.
- **Collected format-mismatch samples and counts:** [`docs/claimed_at_format_mismatch_comparison.md`](claimed_at_format_mismatch_comparison.md), sections 1–6.
- **External-path attribution:** [`docs/claimed_at_root_cause_analysis.md`](claimed_at_root_cause_analysis.md), sections 2–4.

### Interpretation rule for future samples

Classify a value as RFC3339 only when it has a complete date, time, and
timezone and `DateTime::parse_from_rfc3339()` accepts it. Label the first
contract difference explicitly—such as **missing timezone**, **space separator**,
**date only**, or **wrong representation (Unix epoch)**—and retain the raw
sample and source path beside the classification.
