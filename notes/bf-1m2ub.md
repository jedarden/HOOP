# bf-1m2ub — Remove unused utoipa::ToSchema imports

**Date:** 2026-07-09
**Result:** ✅ VERIFIED — no unused `utoipa::ToSchema` imports remain. Task already complete;
prior beads removed every genuinely-dead import. No source changes required.

## Acceptance criteria

| Criterion | Status |
|---|---|
| All unused `utoipa::ToSchema` imports removed | ✅ PASS (none left to remove) |
| `cargo clippy` shows zero unused warnings for utoipa imports | ✅ PASS (`--workspace`, exit 0, zero `utoipa` mentions) |
| `cargo build` succeeds after cleanup | ✅ PASS (`-p hoop-daemon`, exit 0) |
| No broken functionality | ✅ PASS (lib + bins compile & link) |

## Outcome

This bead produced **no `.rs` source changes** — every remaining `utoipa::ToSchema` import in the
workspace is genuinely used. The cleanup this bead targets was already landed by prior beads
(`bf-45ygx`, `bf-4efq2`, `bf-40j9b`, `bf-1aby7`, et al.). This file documents an independent
re-verification rather than duplicating those commits.

## What was checked (fresh runs this session)

1. **Workspace-wide import inventory.** Exhaustive `grep -rn "use utoipa::ToSchema" --include="*.rs"`
   (excluding `target/`, covering src + tests + benches + examples):

   ```
   hoop-daemon: 11 imports
   hoop-mcp:     0
   hoop-schema:  0
   hoop-cli:     0
   hoop-ui:      0
   ```

   All 11 remaining imports live in `hoop-daemon`: `api_audit`, `api_backup`, `api_bead_blockers`,
   `api_bulk_create`, `api_draft_queue`, `api_embedding`, `api_fix_patterns`,
   `api_reflection_detection`, `api_risk_patterns`, `api_stitch_links`, `api_transcription`.

2. **Static use-check (bare vs fully-qualified).** Each of those 11 files was confirmed to actually
   *consume* the imported bare `ToSchema` name in at least one `#[derive(... ToSchema ...)]` (i.e.
   not solely the fully-qualified `derive(utoipa::ToSchema)` form that needs no import). Every file
   has ≥1 bare derive, so every import is used. The dead imports (files whose derives were all
   fully-qualified `utoipa::ToSchema`) were already stripped by the prior commits.

3. **`cargo clippy --workspace`** (nix-shell, default features incl. `openapi`):
   **exit 0**. `grep -ic utoipa` on the log → **0**. No `unused_import` warning anywhere references
   utoipa. The 6 `unused_import` lints that *do* appear are all unrelated — `serde_json::json`
   (`prompt_substitute.rs`), `std::env` (`hoop-cli/config.rs`), `serde::{Deserialize,Serialize}`
   (`hoop-cli/patterns.rs`), `clap::{ArgGroup,Args,Parser,Subcommand}` + `std::io::Write`
   (`hoop-cli/skills.rs`), `serde::Serialize` (`hoop-cli/main.rs`) — out of scope for this bead.

4. **`cargo build -p hoop-daemon`** (nix-shell): **exit 0**, `Finished dev profile … in 15.07s`.
   14 warnings, none utoipa.

## Scope note: `--no-default-features` is not a supported config

A `--no-default-features` build is **pre-existing broken and utoipa-unrelated**: it disables the
`openapi` default feature, after which dozens of unconditional `#[derive(... ToSchema)]` sites and
`T: utoipa::ToSchema` trait bounds fail to compile (`E0432` cannot-find-derive cascades, `E0277`
trait-bound errors on `AgentSessionStatus`, `AgentSessionRow`, `api_beads::*`, …). The codebase is
written assuming `openapi` is on (it is in `default = ["openapi"]`), so the canonical build is
default features — which is clean. The `--no-default-features` breakage is structural drift, not an
unused-import problem, and is out of scope.

## Commands used

```bash
grep -rn "use utoipa::ToSchema" --include="*.rs" . | grep -v /target/   # 11, all used
nix-shell --run 'cargo clippy --workspace'          # exit 0, 0 utoipa mentions
nix-shell --run 'cargo build -p hoop-daemon'        # exit 0
```

The compiler is the authority: `unused_import` is a rustc lint surfaced by both `build` and `clippy`,
so zero `utoipa` mentions across a full `--workspace` clippy proves no unused utoipa imports remain.

## Independent re-verification (re-dispatch run)

Re-dispatched after the original run committed this file but did not push/close. Re-ran every check
from scratch; conclusions are identical — no source changes required.

- `grep -rn "use utoipa::ToSchema" --include="*.rs"` → **11**, all in `hoop-daemon`, each consumed by
  a bare `#[derive(... ToSchema ...)]`. No grouped `use utoipa::{ToSchema, ...}` forms exist anywhere.
- `utoipa` appears only in `hoop-daemon/Cargo.toml`; `hoop-{cli,mcp,schema,ui}` carry zero utoipa imports.
- `cargo clippy --workspace` (nix-shell, default features) → exit 0, `grep -ic "utoipa\|ToSchema"` on the
  full log → **0**. The only `unused import` lints are non-utoipa (`json`, `std::env`,
  `serde::{Deserialize,Serialize}`, `clap::{ArgGroup,Args,Parser,Subcommand}`, `std::io::Write`,
  `serde::Serialize`) — out of scope.
- `cargo build -p hoop-daemon` → `BUILD_EXIT=0`, `Finished dev profile … in 0.19s` (14 warnings, none utoipa).
- `cargo clippy -p hoop-daemon --no-default-features` reconfirmed **pre-existing broken** (E0432
  cannot-find-`ToSchema`-derive cascades + E0277 trait-bound errors on `AgentSessionStatus`,
  `AgentSessionRow`, `api_beads::*` …) — structural drift from `default = ["openapi"]`, not an
  unused-import issue, out of scope.

All four acceptance criteria pass.
