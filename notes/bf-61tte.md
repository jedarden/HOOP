# bf-61tte — Fix utoipa::ToSchema warnings (if found)

## Result: NO WARNINGS FOUND — nothing to fix

## Investigation
The parent bead (bf-2c7x0, dated 2026-07-04) documented 13 files with unused
`use utoipa::ToSchema;` imports. This bead was to remove those unused imports.

Rather than trust the 5-day-old doc, I ran the authoritative check:

```bash
nix-shell --run 'cargo check -p hoop-daemon'   # exit 0
```

## Findings
- **Zero** `utoipa::ToSchema` / `utoipa` unused-import warnings in the current
  `cargo check -p hoop-daemon` output.
- The only unused-import warning is `json` in `prompt_substitute.rs:15` — **out
  of scope** for this bead (not a ToSchema import).
- No crate-level `#![allow(...)]` or `#[allow(unused_imports)]` exists anywhere
  in `hoop-daemon/src/`, so the result is not a false negative.
- There are currently **27** files importing `use utoipa::ToSchema;`. Every one
  of the 16 files named in the parent doc genuinely uses `ToSchema` via
  `#[derive(ToSchema)]` (spot-checked: 1–12 derives per file).

## Why they disappeared
Between bf-2c7x0 (Jul 4) and now (Jul 9), the codebase evolved so that structs
in those files gained `#[derive(ToSchema)]`, making the previously-unused
imports live again. The documented warnings are stale.

## Acceptance criteria
- [x] All documented utoipa::ToSchema warnings fixed — none exist (already resolved)
- [x] `cargo check -p hoop-daemon` passes (exit 0, 14 unrelated pre-existing warnings)
- [x] No new errors introduced — no code changed

## Action taken
No source changes (nothing to remove without breaking a live `#[derive(ToSchema)]`).
This note is the sole artifact of the bead.
