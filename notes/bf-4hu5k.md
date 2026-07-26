# bf-4hu5k — Log retention window vs `<1GB/month`; malformed-bead-line WARN spam

**Type:** investigation (explore)
**Date:** 2026-07-26
**Verdict:** The two questions in this bead are **one problem**. The
malformed-bead-line WARN is not steady-state noise — it is a **real
deserialization bug (schema drift)** between HOOP's `Bead` struct and the
actual `br`/bead-forge `issues.jsonl` wire format. It quarantines **100% of
bead lines** in affected workspaces and accounts for essentially the entire
log volume. Fixing the parse bug is the single high-leverage lever for the
`<1GB/month` target; tightening the retention window is a backstop, not a
solution.

---

## TL;DR

| Question | Answer |
|---|---|
| Is the WARN spam expected noise or a real bug? | **Real bug.** HOOP's `Bead` struct does not match what `br` writes. Every line fails to deserialize. |
| How bad is it? | `694,005` "Quarantined malformed bead line" WARNs in **one day** (2026-07-23) ≈ 91 MB — ~100% of that day's log. |
| Does the retention window alone fix `<1GB/month`? | **No.** At observed volume even 7-day retention ≫ 1 GB. The volume must drop. |
| What drops the volume? | Fixing the parse bug → WARNs go to ~0 → logs drop to the genuine INFO baseline (likely <1 MB/day). |
| Functional impact beyond logs? | **HOOP loads zero beads** from affected workspaces (`bead-forge`, `mobile-gaming`). The bead read path is broken for them, not just noisy. |

---

## Part 1 — Log retention vs the `<1GB/month` target

### Observed disk usage (`~/.hoop/logs/`)

- **2026-07-22 (plan §872):** 26 G across the daemon's first 13 days (started
  2026-07-09) → ~2 GB/day at that churn level. 14-day cutoff had not fired yet
  (nothing was old enough to prune).
- **2026-07-26 (this bead):** 281 M across 4 full days (07-22 → 07-25) plus a
  partial 07-26 → ~73 MB/day average at current churn. Still ~100% WARN spam.

Both snapshots are dominated by one log line type (see Part 2).

### The math error in the existing docs

`docs/operations.md` states (twice):

> 14 days retention × ~100 MB/day = ~1.4 GB max (worst case)
> Log rotation: 100MB/day × 14 days = 1.4GB max

This is **wrong**. `MAX_FILE_SIZE = 100 MB` (`hoop-daemon/src/log_rotation.rs:18`)
is a **per-file** cap, not a per-day volume cap. Rotation opens a new file on
**either** 100 MB **or** 24 h, whichever comes first (`log_rotation.rs:156`).
If write volume is high, the daemon produces many 100 MB files per day — which
is exactly what happened (26 G / 13 d ≈ 20 files/day at the peak). There is no
"100 MB/day" bound anywhere in the code. Corrected in `operations.md` by this
bead.

### Levers

1. **Reduce write volume (primary).** Eliminating the WARN spam removes ~100%
   of current volume. This alone gets logs well under target. See Part 2.
2. **Shorten `MAX_AGE_DAYS` (backstop).** Currently hardcoded `14`
   (`log_rotation.rs:19`, duplicated in `hoop-mcp/src/log_rotation.rs:17`).
   With the parse fix applied this is moot; without it, even `7` days ≫ 1 GB
   at peak churn. It is not configurable — `MAX_AGE_DAYS`/`MAX_FILE_SIZE` are
   not surfaced in `HoopConfig` (`hoop-schema`).
3. **De-duplicate the WARN (defense in depth).** Even after the schema fix,
   any future legitimately-malformed line re-warns on every file rewrite (see
   mechanism below). Rate-limiting (warn-once-per-file-per-session, or
   downgrade repeat quarantines to `debug`) prevents a recurrence. Recommended
   regardless of the schema fix.

### Recommendation (retention)

Do **not** chase `<1GB/month` by shrinking the window alone — the volume is the
problem. Land the schema fix first, then re-measure. Keep `MAX_AGE_DAYS` at a
modest value (7) and make it configurable as a secondary guard. This bead
corrects the documentation but does **not** change the window or the
constants (see Scope).

---

## Part 2 — "Quarantined malformed bead line" WARN spam: root cause

### Where it is emitted

`hoop-daemon/src/beads.rs:285`, inside `BeadReader::parse_all`, wrapping the
`Quarantined` variant of `parse_jsonl_safe::parse_line::<Bead>`:

```rust
crate::parse_jsonl_safe::ParseResult::Quarantined => {
    warn!("Quarantined malformed bead line {} in {}", idx + 1, file_path.display());
}
```

### Why it spams (the multiplier)

`parse_all` is called on **every** read — both `replay_file` (startup) and
`read_updates` (incremental). `read_updates` seeks to the last-read offset, so
already-seen lines are not re-warned — **unless the file is rewritten**. When
`br` rewrites `issues.jsonl` (auto-flush on every `br create/update/claim/close`
per the global CLAUDE.md, plus any `br sync --flush-only` / `br doctor`), the
`notify` watcher fires a Modify/Create event; `FilePosition::is_rotated`
(`beads.rs:66`, size shrinks below offset OR mtime decreases) resets the offset
to 0, and `parse_all` re-parses the **entire** file, re-warning on every line.

So: **N malformed lines × M rewrites = N×M WARN entries.**

Evidence from `hoop.2026-07-23.log`:
- `line 1 in bead-forge` warned **874 times** that day → ~874 full re-parses
  of `bead-forge/.beads/issues.jsonl` in 24 h (≈ one every ~100 s).
- bead-forge: 520,318 WARNs; mobile-gaming: 173,687 WARNs.
- Total 694,005 WARNs ≈ 95,551,810 bytes → **137 bytes/WARN**, i.e. ~100% of
  the day's 91 MB log is this single message.

### The actual bug: schema drift

It is not "a few bad lines". **Every** line in both workspaces is quarantined
(line 1, 2, 3 … through 1487). HOOP's `Bead` struct (`hoop-daemon/src/lib.rs:170`)
does not match the real `br` JSONL format. There are **four** independent
mismatches; any one is sufficient to fail deserialization:

**Real `br` line** (`bead-forge/.beads/issues.jsonl`, line 1, top-level keys):
`acceptance_criteria, assignee, close_reason, closed_at, closed_by_session,
compaction_level, created_at, description, design, id, issue_type, labels,
notes, priority, source_repo, status, title, updated_at`

| # | HOOP `Bead` field | Real `br` value | Problem |
|---|---|---|---|
| 1 | `created_by: String` (required, no default) | **absent** in 840/1487 lines | serde errors on missing required field |
| 2 | `dependencies: Vec<String>` (required, no default) | **absent** in 840/1487 lines | serde errors on missing required field |
| 3 | `status: BeadStatus` enum `{Open, Closed}` — no `rename_all`, no `#[serde(other)]` | lowercase: `closed`(1231) `blocked`(229) `completed`(5) `done`(1) `open`(21) | expects PascalCase `"Open"`/`"Closed"`; also `blocked`/`completed`/`done` have **no** variant |
| 4 | `issue_type: BeadType` enum `{Task,Bug,Epic,Genesis,Review,Fix}` — no `rename_all`, no `#[serde(other)]` | lowercase: `task`(1271) `epic`(145) `bug`(27) `chore`(13) `feature`(10) `test`(8) `genesis`(1) `docs`(1) `story`(1) | expects PascalCase; also `chore`/`feature`/`test`/`docs`/`story` have **no** variant |

Timestamps (`2026-07-22T12:34:55.367115586Z`, 9-digit nanos) are **not** a
failure cause — `chrono::DateTime<Utc>` deserialization accepts up to 9
fractional digits. Extra keys (`design`, `labels`, `assignee`, `closed_at`, …)
are harmless — serde ignores unknown fields by default.

Note mismatch #3 alone fails **100%** of lines (status is always lowercase),
so even the 647 lines that *do* carry `created_by`/`dependencies` still
quarantine. That is why the WARN distribution covers line 1 → 1487 uniformly.

### Functional impact (worse than "noisy logs")

Because every line is quarantined, `parse_all` returns an **empty** `Vec<Bead>`
for these workspaces. HOOP's bead view for `bead-forge` and `mobile-gaming` is
silently empty. This is not just log hygiene — **the core read path is
broken** for any workspace whose `br` emits the modern format shown above.

---

## Recommendations (for a follow-up implementation bead)

This is an explore bead; the code fix is intentionally **not** landed here (it
touches core shared types and must be verified under the Phase 1 build gate —
see Scope). Concrete recipe for the implementation bead:

1. **Fix the schema** in `hoop-daemon/src/lib.rs`:
   - `Bead`: add `#[serde(default)]` to `created_by` and `dependencies` (make
     `created_by: Option<String>` or `String` with `default`).
   - `BeadStatus`: `#[serde(rename_all = "snake_case")]` + add
     `Blocked`/`Completed`/`Done` variants (or a catch-all `#[serde(other)]`
     `Unknown`) to cover `blocked`/`completed`/`done`.
   - `BeadType`: `#[serde(rename_all = "snake_case")]` + add
     `Chore`/`Feature`/`Test`/`Docs`/`Story` (or `#[serde(other)]` `Unknown`).
2. **Add a regression test** that deserializes a real captured `br` line
   (e.g. the line-1 fixture above) into `Bead` and asserts it parses.
3. **De-dup the WARN** in `beads.rs::parse_all`: warn once per `(file, line)`
   per reader session (a `HashSet<(PathBuf, usize)>`), or downgrade repeat
   quarantines to `debug`. Prevents any future legitimate quarantine from
   re-spamming across rewrites.
4. **Make retention configurable**: surface `MAX_AGE_DAYS`/`MAX_FILE_SIZE` in
   `HoopConfig` (hoop-schema) instead of two hardcoded copies (`hoop-daemon`
   + `hoop-mcp`).
5. **Re-measure** `~/.hoop/logs/` after the fix; confirm daily volume drops to
   the INFO baseline and the `<1GB/month` criterion holds.

---

## Scope (what this bead did and did not do)

**Did:**
- Confirmed the WARN spam is a real parse bug (schema drift), not noise.
- Identified the exact field/type mismatches and the volume multiplier.
- Corrected the per-file-vs-per-day math error in `docs/operations.md`.
- Updated `docs/plan/plan.md` §872 to record the confirmed root cause.

**Did not** (deliberately, per AGENTS.md Phase-1 lock + "never claim verified
without `cargo test`"):
- Change `Bead`/`BeadStatus`/`BeadType` or any code.
- Change `MAX_AGE_DAYS` / `MAX_FILE_SIZE`.
- Touch the WARN de-dup logic.

The implementation work is left to a dedicated code bead with tests under the
Phase 1 build gate.
