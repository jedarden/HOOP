# bf-4mrk7 — `hoop reflection export` → Claude Code memory index

## Status: COMPLETE (verified, bead closed)

This bead was dispatched to this session as a **retry** (`failure-count:1`,
`deferred`). The implementation itself was already produced by a prior session
in commit **`c8407d9`** (`feat(bf-4mrk7): add 'hoop reflection export' …`),
which is present on `origin/main`. That prior session committed and pushed the
code but did **not** run `br close`, so NEEDLE recorded the attempt as a failure
and re-dispatched. This session's job was therefore **verification + close**,
not re-implementation.

## What was verified this session

- **Compiles cleanly.** `cargo test -p hoop --bin hoop reflection` builds the
  `hoop` binary (which pulls in `hoop-daemon` as a lib dep) with zero errors.
  (Note: `shell.nix` is currently broken — it pins `nodejs_20`, removed from
  nixpkgs on 2026-04-30 EOL — so bare `cargo` was used directly. That works
  here because hoop-cli's deps avoid the usual NixOS pain: `reqwest` with
  `rustls-tls` and `rusqlite` with `bundled`.)
- **All 10 unit tests pass** (`hoop-cli/src/reflection.rs::tests`):
  slug derivation, title/hook truncation, index-line format, memory-file
  frontmatter + body, YAML escaping of descriptions, add/update/skip planning,
  append idempotency, in-place replace, the full write→skip→update end-to-end
  tempdir cycle, and source-stitch parsing.
- **CLI surface confirmed** at runtime:
  - `hoop reflection --help` lists the `export` subcommand.
  - `hoop reflection export --help` shows `--format claude-memory` (default),
    `--out <DIR>`, `--addr`, `--dry-run`.
- **Struct alignment.** The `ReflectionLedgerEntry` fixture (14 fields) matches
  the live struct at `hoop-daemon/src/fleet.rs:4212` field-for-field, and serde
  uses snake_case verbatim (no `rename_all`), so the consumer's
  `ReflectionsResponse{reflections,count}` deser matches the daemon's
  `GET /api/reflections` (approved-only) response exactly.

## Acceptance criteria — all met

| Criterion | How it's met |
|-----------|--------------|
| `hoop reflection export --dry-run` prints entries that would be added | `run_export` dry-run branch prints each non-Skip entry's file/index/body and returns before any write. |
| `hoop reflection export` appends new entries idempotently | Local export log `<out>/.hoop-reflection-export.jsonl` keyed by `id` + `content_hash` classifies Add/Update/Skip; index append is link-dedup'd. `end_to_end_idempotent_via_tempdir` proves write→skip→update. |
| Unit test with a fixture set | `fixture()` builder + 10 tests. |

## Design notes worth recording

- **Export-only scope honored.** HOOP never deletes or edits existing manual
  memory entries — it appends index lines and rewrites only its own per-entry
  files when content changes.
- **Slug namespacing.** Slugs are prefixed `reflection-<id>` so exported
  entries can never collide with hand-written memory files.
- **Default memory dir** is `~/.claude/projects/-home-coding/memory/`
  (operator's global index), overridable with `--out`.
- The task's scope paragraph suggested "dry-run by default", but the acceptance
  criteria require the *bare* `hoop reflection export` to append idempotently
  (i.e. write by default). The implementation correctly prioritizes the
  acceptance criteria: write is default, `--dry-run` is the opt-in read-only
  path. This is also more useful operationally since the command is already
  non-destructive and idempotent.

## Files touched this session

- `notes/bf-4mrk7.md` (this file) — the implementation itself is unchanged
  from `c8407d9`.
