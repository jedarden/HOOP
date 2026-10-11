# Scope audit — `hoop-daemon/tests/load_test_integration.rs` (hoop-a26c81c8)

- **Date:** 2026-10-11
- **Audited HEAD:** `8dbfd82c416a907a7c04d0e002784cfcbd87a14a` (identical to the split-time HEAD named on the bead)
- **Parent bead:** hoop-72bc70a3 ("Validate the load test integration cleanup", child 1 of 3)
- **Scope of this audit:** the file's diff state and commit history only. No source files were modified.

## Verdict: FLAGGED — 2 of the 3 post-cleanup commits contain hunks beyond import cleanup

Every non-import hunk is a compile-diagnostic fix consistent with its own commit
message and owning bead; nothing is unexplained or behavioral-by-stealth. But the
parent bead's criterion — "confirm the source diff is limited to the requested
import cleanup" — is **not** met for `018ac073` and `16bfcda3`. Details per commit
below.

## 1. Uncommitted-state check — CLEAN

At HEAD `8dbfd82c`, for `hoop-daemon/tests/load_test_integration.rs`:

| Command | Result |
|---|---|
| `git diff HEAD -- hoop-daemon/tests/load_test_integration.rs` | empty (exit 0) |
| `git diff --cached -- hoop-daemon/tests/load_test_integration.rs` | empty (exit 0) |
| `git status --short -- hoop-daemon/tests/load_test_integration.rs` | empty |

No uncommitted or staged edits ride along. The file was last modified on disk
2026-10-03 17:59, matching commit `16bfcda3` (2026-10-03 18:02).

## 2. Per-commit characterization (post-cleanup commits)

`git log --oneline HEAD -- hoop-daemon/tests/load_test_integration.rs` lists 8
commits; the three named on the bead are the three most recent and the only ones
after the import cleanup landed.

### `d7a8bbb1c4d98377b025862277d079dd1e53d979` — 2026-10-03 09:29 — `wip(hoop-59531331): remove unused std path imports`

**Import-cleanup-only for this file.** Single hunk: deletes
`use std::path::PathBuf;` (1 deletion, 0 insertions). Commit footprint: 3 files,
1 deletion each — the other two (`hoop-daemon/src/net_diff.rs`,
`hoop-daemon/tests/cross_workspace_blockers.rs`) are the identical
`use std::path::PathBuf;` removal (verified). Fully within the cleanup scope of
hoop-59531331.

### `018ac073e154927d411e4a9c90a7180d6ebfebac` — 2026-10-03 13:00 — `wip(hoop-2c473ab5): resolve compiler diagnostics across targets`

**NOT import-cleanup-only.** Two hunks in this file (7 lines changed):

1. `@@ -12,6 +12,8 @@` — adds `use futures_util::StreamExt;`. An import *addition*
   (resolving an unresolved-import diagnostic), not a cleanup.
2. `@@ -403,8 +405,11 @@` (in `setup_load_test_projects`) — **functional**:
   - `path: project_path` → `path: project_path.to_string_lossy().into_owned()`
     (type adaptation `PathBuf` → `String` for the schema field), and
   - three new struct fields on the `ProjectsRegistryProjectsItem::Variant0`
     initializer: `label: None, color: None, redaction: None` (schema
     conformance after the registry struct grew fields).

Commit footprint: 18 files, +168/−137 (e.g. `hoop-daemon/Cargo.toml` +20 —
registers `load_test_integration` / `load_test` as `[[test]]` targets plus two
examples, all `required-features = ["testing"]`; `property_invariants.rs` ±124).
Consistent with the commit's own message and owning bead hoop-2c473ab5
("Address hoop-mcp compiler warnings and blocking errors") — compile fixes, not
cleanup.

### `16bfcda39756e8849d8e3a63d794ed86d0391f47` — 2026-10-03 18:02 — `wip(hoop-5ecfee81): unblock load-test integration check`

**NOT import-cleanup-only.** This commit touches only this file (2 insertions,
2 deletions):

1. `@@ -12,7 +12,7 @@` — `use futures_util::StreamExt;` →
   `use futures_util::{SinkExt, StreamExt};`. Import modification (adds `SinkExt`).
2. `@@ -381,7 +381,7 @@` — **functional**:
   `populate_testrepo(load_config, temp_dir)` →
   `populate_testrepo(load_config.clone(), temp_dir)` (moved-value / borrow fix).

Consistent with its message ("unblock load-test integration check" — the added
`required-features = ["testing"]` from `018ac073` surfaced new diagnostics to
fix).

Observation (not a scope finding): this commit is tagged `wip(hoop-5ecfee81)`,
but hoop-5ecfee81's title is "Fix claimed_at timestamp format parsing". The
commit content is load-test unblocking, presumably a mid-bead wip commit made
while that bead's work ran its test suite. Recorded for provenance only.

## 3. Pre-cleanup history (context, not audited in depth)

| SHA | Date | Change to this file |
|---|---|---|
| `0c4d4c40ecc7cca6fdc8a223e90c65a7dad10538` | 2026-04-29 | file created (+487) — feat(ci) performance budget verification |
| `9cc673e34ce756786f3f66085fce8a2734313a80` | 2026-04-29 | +89 — feat(ui) stuck-count badge |
| `9cbe6cddfce938ddd08522c6e1219453a1f48d12` | 2026-05-27 | +12/−6 — docs(load-test) bead count 300 / Phase 2 gate |
| `037bdd4196578c18783206e0082ba3ace92e5ebb` | 2026-08-01 | bf-11213 docs pass — **duplicate commit** of the next row (same title and timestamp); visible only via `git log --follow` (history simplification hides it in the plain path log); broad commit also touching `.beads/issues.jsonl` |
| `95b2d1d8c45c871b5298a133df1829d93732ef4c` | 2026-08-01 | bf-11213 docs pass, +10/−9 |
| `4f9691a7e7d0fe898383f7ea251884b6dc590b6e` | 2026-08-20 | workspace rustfmt pass, +23/−18 |

## 4. File state at HEAD

- 592 lines; no commits touch it in `16bfcda3..HEAD` (blob unchanged since
  `16bfcda3`).
- Both post-cleanup added imports resolve: `futures-util = "0.3"` is a direct
  dependency (`hoop-daemon/Cargo.toml:41`), and the target is registered as
  `[[test]]` with `required-features = ["testing"]`.
- Parent bead hoop-72bc70a3 already verified at `a4737ad6` (file identical to
  current HEAD): `cargo build --all-targets` exit 0 and
  `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
  exit 0 (run twice). This bead re-ran the targeted compile in a clean
  extraction of HEAD as its close verification (§6).

## 5. What "flagged" means for the validation (parent hoop-72bc70a3)

The import cleanup itself (`d7a8bbb1`, hoop-59531331) landed exactly in scope.
Two later commits then made compile-fix changes to the file beyond import
cleanup. Those changes are the reason the file compiles under
`required-features = ["testing"]` today; they are not regressions of the
cleanup and do not need reverting. Any remaining validation effort should target
compile/runtime behavior of the current file, not diff scope — the diff-scope
question is settled by this audit.

## 6. Close-verification run (2026-10-11, clean extraction of committed HEAD `1940f228`)

Extraction: `git archive HEAD | tar -x` into `/tmp/hoop-a26c81c8-dod-Guxdrl`
(removed after success). No `scripts/definition-of-done.sh` exists, so the Rust
default was adapted as follows:

| Command (inside `nix-shell`) | Result |
|---|---|
| `cargo build --all-targets` | SIGTERM'd at 12:36 by memory-cgroup OOM — the worker scope's `MemoryMax=6G` was exceeded compiling `hoop-daemon` (kernel log: `oom-kill:constraint=CONSTRAINT_MEMCG ... task=rustc ... anon-rss:3331604kB`). Infrastructure limit, not a work-product failure; ~351 crates compiled clean first. |
| `cargo build --all-targets -j 3` | Same OOM, exit 143 (parallel big-crate compiles exceed 6G even at 3 jobs). |
| `cargo build --all-targets -j 1` | **exit 0** (18m58s, zero OOM kills). |
| `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run` | **exit 0** (the audited target compiles at HEAD). |

Repo-wide `cargo test` was deliberately not run: at the same file state the
parent bead (hoop-72bc70a3) recorded pre-existing failures in unrelated targets
(agent_session, auth, capacity, config_watcher, fleet, prompt_substitute,
reflection_detector, unknown_event_sink, vector_index) and a >60 s hang in
`integration_harness::tests::test_spawn_and_shutdown`; HOOP's process-cleanup
rules warn leaked test processes OOM the box. Unrelated-failure evidence is on
the parent bead; repeating the run adds risk, not information.
