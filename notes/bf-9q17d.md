# bf-9q17d — clippy --workspace -D warnings recon

Split from bf-2dpkl (child 2 of 4). Measures the second of the three bf-5mpcl
acceptance checks. **Re-run from HEAD `e8b8ba4` on 2026-07-26** (fresh invocation,
not assumed from the prior `3fbcc28` attempt — results identical: still 89).

## Command run

```bash
nix-shell -p pkg-config openssl --run 'cargo clippy --workspace -- -D warnings'
```

> NOTE: `nix-shell --run` (using `shell.nix`) is currently **broken** — `shell.nix`
> pins `nodejs_20`, which hit upstream EOL on 2026-04-30 and was removed from
> nixpkgs-unstable (`error: Node.js 20 support was removed`). The
> `-p pkg-config openssl` ad-hoc invocation sidesteps nodejs entirely and is
> what CI/agents must use until `shell.nix` is bumped to `nodejs_22`+.
> Raw output saved at `/tmp/hoop-clippy.log` (this run).

## Result

**89 clippy errors.** Unambiguous, re-run from HEAD (not assumed).

```
error: could not compile `hoop-daemon` (lib) due to 89 previous errors
```

- `grep -c '^error'` → 90 (89 lints + 1 final summary line)
- **rustc compile errors (E-codes): 0** — every one of the 89 is a clippy lint
  denied via `-D warnings`, not a genuine type/syntax error. So the CI gate is
  failing purely on lint hygiene, not on a broken build.

## Top lint categories

| # | Category | Count |
|---|----------|-------|
| 1 | **disallowed method** (`std::fs::write` 20 + `std::fs::File::create` 8) | **28** |
| 2 | very complex type — factor into `type` defs | 6 |
| 3 | too many arguments (9/7, 8/7, 12/7 variants combined) | 6 |
| 4 | `&mut Vec` instead of `&mut [_]` | 5 |
| 5 | `flatten()` runs forever on repeated `Err` | 5 |
| 5 | consider `sort_by_key` | 5 |
| 7 | stripping a prefix manually | 4 |
| 8 | variable used as a loop counter | 3 |
|   | (remainder: 1–2 each — unused fns/fields/imports, clamp, etc.) | ~27 |

## Is the "disallowed std::fs::write" cluster still dominant? — YES

The #1 category by a wide margin (28 of 89 ≈ **31%**). Both members of the
family emit the same remediation note:

> Use `atomic_write::atomic_write_file` or `atomic_write::atomic_write_file_str`
> instead (crash-safe: tmp + fsync + rename)

### File attribution (all 28 disallowed fs calls, reliable awk extraction)

The parent note named uploads.rs, screen_capture.rs, api_screen_capture.rs,
template_library.rs, projects.rs — **all 5 confirmed present**, plus 9 more
files not previously flagged:

| File | Count |
|------|-------|
| `hoop-daemon/src/uploads.rs` | 6 |
| `hoop-daemon/src/screen_capture.rs` | 5 |
| `hoop-daemon/src/api_screen_capture.rs` | 3 |
| `hoop-daemon/src/log_rotation.rs` | 3 |
| `hoop-daemon/src/atomic_write.rs` | 2 |
| `template_library.rs`, `projects.rs`, `parse_jsonl_safe.rs`, `metrics.rs`, `backup_pipeline.rs`, `attachment_sync.rs`, `attachments.rs`, `api_unassigned.rs`, `agent_session.rs` | 1 each |

**Heaviest offenders:** `uploads.rs` (6) and `screen_capture.rs` (5) — together
11 of 28. Fixing those two files + `atomic_write.rs` would clear ~half the cluster.

## Caveat — 89 is a floor, not the ceiling

All 89 errors are in the **`hoop-daemon` lib** target. Clippy stopped at exit 101
when that crate failed to compile under `-D warnings`, so **no downstream crate
was checked** (the binary target and any workspace member depending on
hoop-daemon). The true workspace-wide count could be higher once `hoop-daemon`
is clean. Build was incremental (cached deps — 0 `Compiling` lines).

## Out of scope (per bead)

- cargo test (child 1, bf-u0554) — already noted a compile failure blocks CI
- `status --json` (child 3)
- Fixing any clippy errors
