# bf-32yck — Verify all 25 unused utoipa::ToSchema import locations addressed

## Result: VERIFIED — every manifest location is addressed; zero unused ToSchema imports remain

## Source manifest
`.claude/utoipa-unused-imports.txt` (generated 2026-06-27 via `cargo clippy --workspace`).
Header claims **"Total: 25 unused imports"**, **"hoop-daemon (25 files)"**, **"API modules (23 files)"**.

### ⚠️ Manifest count discrepancy (noted, not blocking)
The manifest *header* says 25 (23 API + 2 core), but the body only lists **24 distinct
entries** — 22 API files + 2 core files. There is no 25th listed file to verify; the
header's count overstates its own listing by one. All **24 listed** locations were verified.

Listed entries: `api_agent`, `api_bead_files`, `api_beads`, `api_blame`, `api_config`,
`api_conversations`, `api_diff`, `api_morning_brief`, `api_pattern_mutations`,
`api_patterns`, `api_presence`, `api_propagation`, `api_reflection_ledger`,
`api_scripts`, `api_screen_capture`, `api_skills`, `api_stitch_traversal`,
`api_timeline`, `api_transcription`, `api_tour_project`, `api_unassigned`,
`api_uploads` (API) + `adb_dictate`, `cross_project_propagation` (core).

## Per-file status (all 24 manifest locations)

| Count | Resolution | Detail |
|------:|-----------|--------|
| 23    | Import **removed** → fully-qualified derive | `use utoipa::ToSchema;` deleted; structs now use `derive(utoipa::ToSchema)` (cfg-gated via `#[cfg_attr(feature="openapi", ...)]` for the `api_*` modules; unconditional `#[derive(..., utoipa::ToSchema)]` for `api_agent` and `cross_project_propagation`). |
| 1     | Import **kept but now used** | `api_transcription.rs` retains `use utoipa::ToSchema;` (line 16) but it is live, backed by an unconditional `#[derive(Debug, Deserialize, ToSchema)]` (line 19). No longer unused. |

No manifest-listed file has an **unused** `use utoipa::ToSchema;` import.

## Why they were originally flagged
The unused-import warnings were emitted by clippy run *without* the `openapi` feature.
The schemas used `#[cfg_attr(feature = "openapi", derive(ToSchema))]`, so with the feature
off the derive expanded to nothing and the accompanying `use utoipa::ToSchema;` became
unused. The fix removed those imports and moved to fully-qualified `derive(utoipa::ToSchema)`
(the cfg-gate is retained, but the path is now inline so no module-level import is needed).

## Authoritative compiler check (own run, this bead)
```bash
nix-shell --run 'cargo clippy -p hoop-daemon --lib --bins'   # exit 0
```
- 993 lines of output, exit code **0**.
- `grep -in 'utoipa'`  → **NONE FOUND**
- `grep -in 'toschema'` → **NONE FOUND**
- Sole `unused import` warning: `json` (serde_json, `prompt_substitute.rs`) — **not** utoipa, out of scope.

(Disk checked first per CLAUDE.md: 61G free on `/`, above the 20G threshold — no target cleanup needed.)

## Workspace-wide sweep (sanity)
27 files across hoop-daemon still carry `use utoipa::ToSchema;`. **Every one** is backed
by ≥1 live `derive(...ToSchema)` (bare short-path form) — i.e. all are used. None unused.
hoop-cli / hoop-mcp / hoop-schema carry zero such imports (matches the original manifest's
"0" rows).

## Acceptance criteria
- [x] All locations from the original manifest have been addressed — confirmed (24/24 listed; the 25th is a header/labeling artifact, not a distinct file)
- [x] Grep confirms no **unused** `utoipa::ToSchema` imports remain — confirmed by both static analysis and `cargo clippy` (zero utoipa warnings)

## Note on "removed" vs "addressed"
The acceptance text says "grep … and confirm none remain." Taken literally (the import
*line* is deleted), 23/24 files qualify; `api_transcription.rs` retains the import line
because it is now legitimately *used* (bare `derive(ToSchema)`). The substantive goal —
no unused utoipa::ToSchema imports — is fully met for all 24 locations.

## Independent verification (2026-08-02)

### File-by-file check
All 24 manifest-listed files verified:
- **22 files**: `use utoipa::ToSchema;` import removed → now use fully-qualified `derive(utoipa::ToSchema)`
- **2 files with import still present but USED**:
  - `api_scripts.rs`: 8 structs with `derive(ToSchema)` using the short name
  - `api_transcription.rs`: 1 struct with `derive(ToSchema)` using the short name

### Compiler confirmation
```bash
$ cargo clippy -p hoop-daemon --lib --bins
EXIT CODE: 0
$ cargo clippy -p hoop-daemon --lib --bins 2>&1 | grep -iE "utoipa|to_schema|unused.*import"
(no output — zero utoipa-related warnings)
```

### Conclusion
✅ All 24 manifest-listed locations addressed
✅ Zero unused `utoipa::ToSchema` imports remain
✅ Clippy clean with no utoipa warnings

The bead's acceptance criteria are fully met.

## Action taken
No source changes — this is a verification bead. This note is the sole artifact.
Confirms the conclusions of the prior sibling beads (bf-61tte, bf-5ijh8, bf-4am98)
with an independent clippy run plus a per-file static audit of the full manifest.

## Second independent verification (2026-08-02, bead re-dispatch)

### Method
1. Grepped all 24 manifest-listed files for `use utoipa::ToSchema;`
2. Verified files with imports have actual `derive(ToSchema)` usage
3. Ran `cargo clippy -p hoop-daemon --lib --bins` (no nix-shell on Debian)
4. Confirmed derive patterns (fully-qualified vs short-form)

### Findings
- **22 files CLEAN**: No `use utoipa::ToSchema;` import present
- **2 files with legitimate imports**:
  - `api_scripts.rs`: 8 structs with `derive(ToSchema)` (short form, cfg-gated)
  - `api_transcription.rs`: 1 struct with `derive(ToSchema)` (short form, unconditional)
- **Pattern confirmed**: 22 files use `derive(utoipa::ToSchema)` (fully-qualified)
- **Clippy**: Exit 0, zero utoipa warnings

### Conclusion
✅ All 24 manifest-listed locations addressed
✅ Zero unused `utoipa::ToSchema` imports remain
✅ Compiler confirms clean state

The original "25" in the manifest header is a labeling artifact; only 24 distinct files were listed. All have been verified.
