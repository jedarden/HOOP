# Targeted load-test integration compile

## Verification at committed HEAD (2026-10-12, attempt 6)

- HEAD: `3a10b86d934c92407f2897515d5d24af508b3198`
- Command: `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
- Exit code: `0`
- Extraction: clean `git archive HEAD` copy at `/home/coding/scratch/hoop-53fc6b3b-load2.attempt6` (removed after verification; captured log kept, see below)
- Build path: `nix-shell --run` on NixOS with nix-provided Cargo 1.98.1 and `CARGO_TARGET_DIR=/build/target` (the warm shared cache this bead's runs use); the `~/.local/bin/cargo` wrapper was not invoked — nix-shell shadows it — so this was local execution, not an iad-ci submit. Same methodology as the earlier recorded runs on this bead.
- Result: the `load_test_integration` test target compiles at current committed HEAD. `Finished … in 0.44s` means the shared cache fingerprint-matched this exact HEAD's bit-identical sources; the invocation itself was a fresh `--no-run` compile check in a fresh extraction.

Captured output tail:

```text
warning: `hoop-daemon` (test "load_test_integration") generated 6 warnings (run `cargo fix --test "load_test_integration" -p hoop-daemon` to apply 5 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.44s
  Executable tests/load_test_integration.rs (/build/target/debug/deps/load_test_integration-7a989f4ddf608c3a)
TARGETED_EXIT_CODE=0
```

Full captured output retained at `/home/coding/scratch/hoop-53fc6b3b-load2-attempt6.log`.

## Definition-of-done status at this HEAD (why the bead stays open)

The dispatch's mandatory pre-close checklist requires the language-default
definition of done — `cargo build --all-targets && cargo test` — to pass in a
clean extraction. It does not pass at `3a10b86d`, for reasons outside this
bead's scope (this bead must not modify source files):

- Probe at `3a10b86d` in the same clean extraction:
  `cargo test -p hoop-daemon --lib vector_index` → **exit 101**,
  `test result: FAILED. 3 passed; 5 failed; 0 ignored` (finished in 16.57s).
  Failing: `test_index_rebuild_and_search`, `test_cross_project_duplicate`,
  `test_synthetic_cross_project_recall`, `test_description_enhances_matching`,
  `test_performance_10k_items_p95_under_50ms` — assertion failures in the
  similarity/dedup matching itself (e.g. `check_duplicate` returns no match),
  deterministic when run in isolation. Probe log retained at
  `/home/coding/scratch/hoop-53fc6b3b-vector-probe.log`.
- The same five failed in the full-suite run at `5b23698c`
  (`/home/coding/scratch/hoop-53fc6b3b-dod-final.log`, which the attempt's own
  timeout ended at `DOD_EXIT_CODE=124` while the suite was still running).
  Between `5b23698c` and `3a10b86d` no Rust source changed (`fd233caa` touched
  only `bin/test_categorize_error_messages.py`; `3a10b86d` is docs-only), so
  the failure set at HEAD is exactly these five.
- Owning bead for the failing tests: `hoop-bb0dad4d` ("Fix failing tests",
  InProgress). No duplicate bead created.

This bead's own acceptance criteria — the targeted compile check, its exact
command, HEAD SHA, exit code recorded on the bead, and this committed
artifact — are met at current HEAD. The bead is left open solely because the
close gate (full dod) cannot pass until `hoop-bb0dad4d` lands; the next
attempt can close by re-running the check and the dod in a clean extraction
once that bead's work is committed.
