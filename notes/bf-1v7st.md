# bf-1v7st — Add ToSchema derives (api_scripts, api_tour_project, api_transcription)

## Outcome: verified — all three already carry working `ToSchema` derives; clean build, zero errors

Inspected all three target structs and confirmed each already derives
`ToSchema`. The build is clean: **0 errors** (better than the ~39 the task
brief anticipated). No code change was needed — this bead landed as a
verification in the same vein as the recent utoipa beads
(bf-45ygx, bf-49sn9, bf-32yck, bf-1aby7, bf-1m2ub, bf-2oynm, bf-18mvt, bf-3g946 …).

### Acceptance criteria — all met (one exceeded)

1. **`ScriptRunRequest` has `#[derive(ToSchema)]`** — yes.
   `hoop-daemon/src/api_scripts.rs:162` — `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`.
2. **`EnableTourRequest` has `#[derive(ToSchema)]`** — yes.
   `hoop-daemon/src/api_tour_project.rs:34` — `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]`.
3. **`ListJobsQuery` has `#[derive(ToSchema)]`** — yes.
   `hoop-daemon/src/api_transcription.rs:19` — `#[derive(Debug, Deserialize, ToSchema)]` (via `use utoipa::ToSchema;` at line 16).
4. **All non-ToSchema field types handled** — yes; the build would fail
   otherwise. Fields are all `Option<String>` / `Vec<String>` (types
   utoipa supports natively).
5. **`cargo check` errors reduced to ~39** — *exceeded*. Actual error count
   is **0**. The ~39 premise reflected an outdated repo state; the prior
   utoipa-import-cleanup beads already resolved those. The lib now compiles
   with no errors at all.

### Two derive forms are both legitimate here

The three structs use two different, both-valid conventions that coexist
throughout `hoop-daemon/src/`:

| Struct | File:line | Derive form |
|--------|-----------|-------------|
| `ScriptRunRequest` | `api_scripts.rs:162` | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
| `EnableTourRequest` | `api_tour_project.rs:34` | `#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]` |
| `ListJobsQuery` | `api_transcription.rs:19` | unconditional `#[derive(Debug, Deserialize, ToSchema)]` |

The `cfg_attr` form is the dominant convention (46 files). The direct
`use utoipa::ToSchema;` + `#[derive(..., ToSchema)]` form is also widespread
(api_embedding, api_audit, api_bulk_create, api_draft_queue, …), so
`ListJobsQuery` is consistent with its sibling files — not an inconsistency
to fix. Its `use utoipa::ToSchema;` import is genuinely used (no unused-import
warning), unlike the stale imports the earlier cleanup beads removed.

### Why the `cfg_attr` derives were actually compiled (not skipped)

`ScriptRunRequest` and `EnableTourRequest` gate `ToSchema` behind
`#[cfg_attr(feature = "openapi", ...)]`. That feature is **on by default** in
`hoop-daemon/Cargo.toml` (`default = ["openapi"]`), so a plain `cargo check`
*does* expand the derives — they were genuinely type-checked, not cfg'd out.

### Airtight cross-check: the OpenAPI generator binary

`cargo check --bin generate_openapi` — the target that actually *consumes*
the `ToSchema` derives via `#[derive(utoipa::OpenApi)]` — also compiles with
exit code `0`, 0 errors, no utoipa/ToSchema diagnostics. If any of the three
derives were malformed, this is the target that would fail.

### Warnings (14, all unrelated)

The 14 warnings are entirely dead-code notices in `capacity.rs`,
`sessions.rs`, `stitch_percentile_index.rs`, and prompt parsing. A grep of
the full `cargo check` output for `utoipa`/`toschema` returns **nothing**, and
none of the three struct names appear in any diagnostic.

### Verification commands

```bash
nix-shell --run 'cd hoop-daemon && cargo check --lib'              # exit 0, 0 errors
nix-shell --run 'cd hoop-daemon && cargo check --bin generate_openapi'  # exit 0, 0 errors
```
