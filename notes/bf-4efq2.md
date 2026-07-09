# bf-4efq2 — Remove unused utoipa imports from hoop-cli

## Outcome
**No changes required.** The hoop-cli crate (package name `hoop`, located at
`hoop-cli/`) contains **zero** `utoipa` references — there were no unused
`utoipa::ToSchema` imports to remove.

## Evidence

- **`utoipa` is not a dependency** of hoop-cli — it does not appear in
  `hoop-cli/Cargo.toml`'s `[dependencies]` (or `[dev-dependencies]`).
- **No source references:** `grep -rn "utoipa" hoop-cli/` (excluding `target/`)
  returns no matches across `src/` and `tests/`.
- **Never present in history:** `git log -S "utoipa" -- hoop-cli/` is empty,
  so utoipa was never added to this crate.
- **Build passes:** `cargo build -p hoop` succeeds (package name is `hoop`,
  not `hoop-cli`).
- **Clean clippy:** `cargo clippy -p hoop` produces **zero** utoipa-related
  warnings.

## Notes
The prior utoipa cleanup beads in this series targeted `hoop-daemon`. hoop-cli
relies on `hoop-daemon` and `hoop-schema` for its utoipa/OpenAPI types, so it
never needed its own `utoipa::ToSchema` imports.
