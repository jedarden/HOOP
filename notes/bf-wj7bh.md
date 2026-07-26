# Bead bf-wj7bh: serde defaults for created_by/dependencies (bf-315nx mismatch #1/#2)

## Summary

The bf-wj7bh change was **already committed in `3a49209`** by a prior dispatch of
this same bead (re-dispatched with `failure-count:1`; it was never closed). This
note documents the change and the verification performed on this dispatch.

**Scope:** `hoop-daemon/src/lib.rs` — the `Bead` struct ONLY.
- `created_by: String` now carries `#[serde(default)]` → empty `String` when the key is absent.
- `dependencies: Vec<String>` now carries `#[serde(default)]` → empty `Vec` when the key is absent.
- Field types unchanged (`String` / `Vec<String>`); no other field or behavior touched.

`br`/bead-forge omits these two keys on many beads (840/1487 captured lines),
which previously made `serde_json::from_str::<Bead>(…)` fail with a missing-field
error and quarantine the whole line.

## Verification

The hoop-daemon lib does **not** currently compile end-to-end — there are
pre-existing Phase-1-in-progress errors (missing fields in `DaemonState`,
`CapacityMeterConfig`, `HoopConfig`, `PreviewRequest`, `NeedleEvent`, etc.; a
`ProjectSupervisor::new` arity mismatch; missing `ResolvedConfig::default`).
AGENTS.md documents this state explicitly ("`cargo build` FAILS … the crate does
not currently compile"). **None** of the 31 `lib test` errors touch `Bead`,
`created_by`, or `dependencies`, so the crate-wide breakage is unrelated to this
bead and not fixable within its narrow scope.

To verify the bf-wj7bh serde semantics concretely, a throwaway standalone crate
(`~/scratch/bf-wj7bh-verify`, since removed) mirrored the **exact** committed
field declarations of `Bead` — including `BeadStatus { Open, Closed }` /
`BeadType { Task, Bug }` with **no** `rename_all`, matching the committed state
exactly (lowercase-status handling is bf-315nx mismatch #3, out of scope here) —
and exercised both acceptance criteria against real `serde`/`serde_json`:

```
[1] PASS: omitted created_by/dependencies default cleanly (no missing-field error)
[2a] PASS: present created_by/dependencies round-trip to actual values
[2b] PASS: each key defaults independently of the other
[3] PASS: explicit empty values handled identically to defaults
[4] PASS: Bead with created_by/dependencies round-trips through serde_json
```

Acceptance criteria met:
- ✅ A line omitting both keys deserializes into `Bead` with no missing-field error
  (`created_by == ""`, `dependencies.is_empty()`).
- ✅ A line including them deserializes to their actual values — defaults do not
  clobber present data, and each key defaults independently.
- ✅ No other deserialization behavior changed (only two `#[serde(default)]`
  attributes were added; serialized round-trips are unchanged).

## Scope boundary (important)

The working tree currently has **uncommitted** changes to `lib.rs` that expand
`BeadStatus`/`BeadType` (`#[serde(rename_all = "snake_case")]`, new variants
`Blocked/Completed/Done` and `Chore/Feature/Test/Docs/Story`, plus an
`#[serde(other)] Unknown` catch-all) and add a `bead_schema_tests` module. Those
are **bf-315nx scope (mismatches #3 & #4)** — explicitly out of scope for
bf-wj7bh, which says "Do NOT change … any other field" and "No other
deserialization behavior changed." They were **left untouched** (not committed
under this bead and not discarded) for the parent umbrella bead to pick up.

The committed bf-wj7bh change (`3a49209`) is intentionally minimal and
self-contained: in isolation it removes the created_by/dependencies
missing-field error but does **not** by itself make a real lowercase-status br
line fully deserialize — that requires the bf-315nx status/type fixes. This
matches the bead's narrow acceptance criteria.
