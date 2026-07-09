# bf-1aby7 — Verify utoipa import cleanup complete

**Date:** 2026-07-09
**Result:** ✅ VERIFIED — all unused utoipa imports removed; zero utoipa warnings; workspace builds clean.

## Acceptance criteria

| Criterion | Status |
|---|---|
| `cargo clippy` shows zero unused-import warnings for utoipa | ✅ PASS |
| `cargo build` succeeds without errors | ✅ PASS (`--workspace`, exit 0) |
| All originally-identified unused imports removed | ✅ PASS (remaining imports all genuinely used) |
| No functionality broken | ✅ PASS (lib + bins compile & link) |

## What was checked

1. **Static import audit.** utoipa is a dependency of `hoop-daemon` only (no other workspace
   member references it — confirmed via `grep utoipa */Cargo.toml`). Of the source files that
   still carry `use utoipa::ToSchema;`, every one was confirmed to actually *use* the imported
   (bare) `ToSchema` name in at least one `#[derive(... ToSchema ...)]`. Files whose derives use
   only the fully-qualified `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` form had
   their dead `use utoipa::ToSchema;` removed by the prior cleanup commits.

   The 11 remaining `use utoipa::ToSchema;` imports (each with a verified bare usage count ≥ 1):
   `api_audit` (7), `api_backup` (2), `api_bead_blockers` (2), `api_bulk_create` (6),
   `api_draft_queue` (14), `api_embedding` (6), `api_fix_patterns` (10),
   `api_reflection_detection` (3), `api_risk_patterns` (9), `api_stitch_links` (4),
   `api_transcription` (1).

2. **`cargo build --workspace`** (nix-shell, default features incl. `openapi`):
   `Finished dev profile ... in 2m 08s`, **exit 0, 0 errors, 0 utoipa mentions** in the log.

3. **`cargo clippy --workspace --all-targets`** (nix-shell): the library crate compiled cleanly
   (its 91 warnings are all pre-existing lints — `lines_filter_map_ok`, etc. — **none utoipa**).
   Grep of the full clippy log for any `utoipa` warning/error: **NONE FOUND**.

## Scope note: pre-existing, utoipa-unrelated test-target errors

`cargo clippy --all-targets` exits 101, but every error is in **test/bench targets** and is
**unrelated to utoipa**:

- `E0432/E0433` — `tempfile` unresolved (97×). `tempfile` is `optional = true`, enabled only via
  the `testing` feature (`testing = ["tempfile"]`). Test code needs `--features testing` to compile;
  the clippy run did not pass it.
- `E0277` — async block `cannot be unpinned` (28×) in `syntax_highlight_stream.rs`.
- `E0063` — struct initializer drift (~18×): `DaemonState` (missing `br_semaphore`,
  `br_semaphore_target_permits`), `PreviewRequest` (`attachments_count`), `CapacityMeterConfig`,
  `DictatedNote`, `HoopConfig`, `NeedleEvent`, `net_diff::CommitEntry`.
- `E0061/E0308` — function arity / type mismatches in test call sites.
- `E0599` — missing `::default` for `ResolvedConfig`, `RedactionPolicyState`, `SecretPattern`.

The 6 utoipa-cleanup commits (`924a75c~1 → HEAD`) made **zero additions to `.rs` source** — they
removed only `use utoipa::ToSchema;` lines (16 lib files × 3 lines = 48 deletions; the only
insertions are these `notes/*.md` docs). Deleting an import cannot introduce a missing struct
field, change function arity, or add a feature requirement, so these test errors are pre-existing
drift, out of scope for this utoipa verification.

## Commands used

```bash
nix-shell --run 'cargo build --workspace'                 # exit 0
nix-shell --run 'cargo clippy --workspace --all-targets'  # lib clean; test-target errors unrelated
grep -niE 'utoipa' /tmp/hoop-build.log                    # NONE
grep -niE 'warning.*utoipa|error.*utoipa' /tmp/hoop-clippy.log   # NONE
```
