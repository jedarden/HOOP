# bf-5ijh8 — Verify clean clippy run on hoop-daemon

## Result: VERIFIED — no utoipa::ToSchema warnings (none of any kind)

## Goal
Final verification that clippy runs clean on hoop-daemon, with specific focus on
confirming no `utoipa::ToSchema` unused-import warnings remain.

## Command run
```bash
nix-shell --run 'cargo clippy -p hoop-daemon'   # exit 0
```
Full output captured to `/tmp/clippy-hoop-daemon.txt` (993 lines).

## Findings

### utoipa / ToSchema — ZERO warnings
- `grep -in 'utoipa'`  over the full clippy output → **NONE FOUND**
- `grep -in 'toschema'` over the full clippy output → **NONE FOUND**
- Not a false negative: no `#![allow(...)]` crate-level attrs and no
  `#[allow(.*utoipa|unused_imports|unused)]` exist anywhere in
  `hoop-daemon/src/`. 27 files import `use utoipa::ToSchema;` and every import
  is kept live by `#[derive(ToSchema)]`.

This confirms the conclusion of the prior bead (bf-61tte): the utoipa::ToSchema
warnings documented in the Jul-4 parent bead are stale and remain resolved.

### Remaining warnings — all non-utoipa (documented)
clippy exits 0 but emits **91 warnings** (warnings, not errors — no `-D warnings`).
Breakdown by type:

| Count | Category | Note |
|------:|----------|------|
| 30 | `disallowed_methods` (`std::fs::write` ×22, `std::fs::File::create` ×8) | project `clippy.toml`/lint config wants `atomic_write::*` (crash-safe tmp+fsync+rename) |
| ~12 | dead code (never-used functions/fields/constants) | e.g. `openapi_router`, `load_hoop_config`, `STITCH_CLOSED_THRESHOLD_SECONDS` |
| 6 | `type_complexity` | `clippy.toml` threshold = 500 |
| 6 | `too_many_arguments` (8/9/12 of 7) | `clippy.toml` threshold = 12 |
| 5 | `unnecessary_sort_by` → use `sort_by_key` |
| 5 | `ptr_arg` (`&mut Vec` → `&mut [_]`) |
| 5 | `lines_filter_map_ok` (`flatten()` → `map_while(Result::ok)`) |
| 4 | `manual_strip` |
| 3 | `explicit_counter_loop` |
| 2 each | `map_identity`, `should_implement_trait`, `manual_clamp`, `unnecessary_unwrap` |
| 1 each | `unused import: json` (serde_json, **not** utoipa), `len_without_is_empty`, `large_enum_variant`, `if_same_then_else`, `filter_next`, `doc_overindented_list_items`, private-type-leak |

The single `unused import` warning is `serde_json::json` in
`hoop-daemon/src/prompt_substitute.rs:15` — **out of scope** (not a ToSchema
import). The `too_many_arguments`/`type_complexity` thresholds are intentionally
relaxed in `clippy.toml` to let Phase 2–7 code (unverified per AGENTS.md) pass.

## Acceptance criteria
- [x] `cargo clippy -p hoop-daemon` produces no `utoipa::ToSchema` unused-import warnings — confirmed (zero utoipa mentions)
- [x] No other utoipa-related warnings — confirmed (zero utoipa mentions)
- [x] Clean clippy output OR only non-utoipa warnings (documented) — 91 non-utoipa warnings documented above

## Action taken
No source changes — this is a verification bead. This note is the sole artifact.
The remaining 91 warnings are pre-existing, non-utoipa, and largely gated by
`clippy.toml` thresholds for unverified Phase 2–7 code; they are out of scope for
this bead and tracked separately (see AGENTS.md Phase 1 exit gate `bf-5mpcl`).

## Re-verification (2026-07-09)
Re-ran `nix-shell --run 'cargo clippy -p hoop-daemon'` from a clean incremental
build — **identical result**: exit 0, `hoop-daemon` (lib) generated **91 warnings**.
- `grep -in 'utoipa'`  → NONE FOUND
- `grep -in 'toschema'` → NONE FOUND
- Sole `unused import` remains `serde_json::json` (`hoop-daemon/src/prompt_substitute.rs:15`) — not utoipa.

Clippy version: rust-1.96.0 (per `rust-clippy/rust-1.96.0/index.html` help links).
Conclusion unchanged: the utoipa::ToSchema warnings remain resolved; acceptance
criteria met. No source changes this run either.

## Final verification (2026-08-02)
Ran `cargo clippy -p hoop-daemon` directly (no nix-shell wrapper needed on Debian).
**Result:** Exit 0, zero output (completely clean).
- No utoipa warnings
- No other warnings
- All acceptance criteria met

This confirms the utoipa::ToSchema fixes remain in place and hoop-daemon passes
clippy cleanly.
