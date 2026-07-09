# bf-18mvt — Verify ToSchema derives with cargo check

## Outcome: verified — clean build, zero errors

Ran `cargo check` to confirm all three structs carry working `ToSchema` derives
and compile. The build is clean: **0 errors** (better than the ~43 the task
brief anticipated).

### Acceptance criteria — all met (one exceeded)

1. **`cargo check` runs successfully** — yes.
   `nix-shell --run 'cargo check --workspace'` → exit code `0`,
   `Finished dev profile ... in 1m 26s`.
2. **Error count ≈ 43 (down from ~46)** — *exceeded*. The actual error count is
   **0**. The ~46→~43 premise reflected an outdated repo state; the prior
   utoipa-import-cleanup beads (bf-45ygx, bf-49sn9, bf-32yck, bf-1aby7,
   bf-1m2ub, bf-2oynm, …) already resolved those. The workspace now compiles
   with no errors at all.
3. **All three structs compile with `ToSchema`** — yes. None of the three
   produced any diagnostic (no error, no warning):

   | Struct | File:line | Derive form |
   |--------|-----------|-------------|
   | `SiblingProject` | `hoop-daemon/src/cross_project_propagation.rs:22` | unconditional `#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]` |
   | `ApproveProposalRequest` | `hoop-daemon/src/api_reflection_ledger.rs:42` | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
   | `RejectProposalRequest` | `hoop-daemon/src/api_reflection_ledger.rs:60` | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |

### Why the `cfg_attr` derives were actually compiled (not skipped)

`ApproveProposalRequest` and `RejectProposalRequest` gate `ToSchema` behind
`#[cfg_attr(feature = "openapi", ...)]`. That feature is **on by default** in
`hoop-daemon/Cargo.toml` (`default = ["openapi"]`, line 7), so a plain
`cargo check` *does* expand the derives — they were genuinely type-checked,
not cfg'd out.

### Airtight cross-check: the OpenAPI generator binary

`cargo check --workspace` already checks every workspace target including
`generate_openapi` (a bin with `required-features = ["openapi"]`), which is the
target that actually *consumes* the `ToSchema` derives via `#[derive(utoipa::OpenApi)]`.
Forced a direct check as well:

```bash
nix-shell --run 'cargo check --bin generate_openapi'
```

→ exit code `0`, 0 errors, no utoipa/ToSchema diagnostics. If any of the three
derives were malformed, this is the target that would fail.

### Warnings (30, all unrelated)

The 30 warnings are entirely `unused variable` / `unused import` style
(`std::io::Write`, `serde::Serialize`, `Deserialize`/`Serialize`, `ArgGroup`/
`Args`/`Parser`/`Subcommand`, etc.). A grep of the full `cargo check` output
for `utoipa`/`toschema` returns **nothing**, and none of the three struct names
appear in any diagnostic.

### Verification commands

```bash
nix-shell --run 'cargo check --workspace'        # exit 0, 0 errors
nix-shell --run 'cargo check --bin generate_openapi'  # exit 0, 0 errors
```
