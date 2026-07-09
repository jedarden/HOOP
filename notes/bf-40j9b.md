# Utoipa Import Cleanup Verification - bf-40j9b

**Date:** 2026-07-09
**Task:** Remove all unused `utoipa::ToSchema` imports from `hoop-mcp` and `hoop-schema` crates.
**Outcome:** NO CODE CHANGES REQUIRED — imports were already removed in prior beads.

## Scope

Target files: `hoop-mcp/src/**/*.rs` and `hoop-schema/src/**/*.rs`.

## Verification

### 1. No utoipa imports exist anywhere in either crate

Full case-insensitive sweep across every `.rs` file (src, tests, build.rs) and both `Cargo.toml` manifests:

```bash
grep -rni "utoipa" hoop-mcp/ hoop-schema/   # (excluding node_modules)
grep -rni "toschema" hoop-mcp/src hoop-schema/src
```

- **hoop-mcp:** zero matches — no `use utoipa` imports, no `ToSchema` trait usage, no dependency in `Cargo.toml`.
- **hoop-schema:** zero `.rs` matches. Only utoipa mentions are in `hoop-schema/README.md` (documentation prose describing how `openapi.yaml` is generated from utoipa annotations in `hoop-daemon`) — out of scope (`src/**/*.rs` only).

### 2. Build succeeds

```bash
nix-shell --run 'cargo build -p hoop-mcp -p hoop-schema'
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.36s   EXIT: 0
```

### 3. Clippy — zero utoipa warnings

```bash
nix-shell --run 'cargo clippy -p hoop-mcp -p hoop-schema --all-targets'   # EXIT: 0
```

No utoipa-related warnings are emitted. The non-utoipa warnings present are pre-existing and out of scope for this bead:

- `hoop-schema` (`schema_drift` test): `clippy::needless_borrows_for_generic_args` (2)
- `hoop-mcp/src/skills.rs:529-530`: unused imports `std::fs::File`, `tempfile::TempDir` (test code)
- `hoop-mcp/tests/forbidden_worker_steering.rs:115,148`: unused `mut`

## Acceptance Criteria

- [x] All unused utoipa imports removed from `hoop-mcp/src/` and `hoop-schema/src/` (none present)
- [x] No utoipa-related clippy warnings remain for these crates
- [x] `cargo build -p hoop-mcp -p hoop-schema` succeeds

## Notes

This bead is a verification sweep in the utoipa-cleanup series (see git log: bf-4efq2, bf-45ygx, bf-49sn9, bf-32yck, bf-4am98). The unused `utoipa::ToSchema` imports had already been removed; both `hoop-mcp` and `hoop-schema` are clean. No file changes were produced.
