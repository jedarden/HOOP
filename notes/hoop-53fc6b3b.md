# Targeted load-test integration compile

## Verification at committed HEAD (authoritative)

- HEAD: `7274c233e01281e5916dddcc5ecafd319bf4ca57`
- Command: `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
- Exit code: `0`
- Extraction: clean `git archive HEAD` copy at `/home/coding/scratch/hoop-53fc6b3b-verify.6eRMXGmY` (removed after successful verification)
- Build path: `nix-shell --run` on NixOS (nix-provided cargo 1.98.1; the `~/.local/bin/cargo` wrapper did not run, so this was a local execution, not an iad-ci submit), with `CARGO_TARGET_DIR=/build/target` — the existing shared cache for this repo; no new target dir was created
- Result: the `load_test_integration` test target compiles at committed HEAD `7274c233`, a committed state that includes `16bfcda3`'s post-`a4737ad6` changes to the target. The produced binary hash (`load_test_integration-7a989f4ddf608c3a`) matches the full compile performed at this same SHA earlier on this bead, so this run is a fingerprint-validated cache hit (0.63s) on top of that compile.

Captured output tail:

```text
warning: `hoop-daemon` (test "load_test_integration") generated 6 warnings (run `cargo fix --test "load_test_integration" -p hoop-daemon` to apply 5 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.63s
  Executable tests/load_test_integration.rs (/build/target/debug/deps/load_test_integration-7a989f4ddf608c3a)
ACCEPTANCE_EXIT_CODE=0
```

## Definition-of-done check (same extraction)

- `cargo build --all-targets` → exit `0`: every workspace target compiles at committed HEAD (full log: `/home/coding/scratch/hoop-53fc6b3b-verify2.log`).
- Full `cargo test` was not re-run: the suite stalls in hoop-daemon integration tests at this HEAD (earlier on this bead it was interrupted at exit 130; retained log `/home/coding/scratch/hoop-53fc6b3b-dod.log`). That is the pre-existing Phase 1 gate condition tracked on `hoop-7ae9fbff`, outside this bead's compile-check scope.

## Prior evidence (superseded HEAD record)

An earlier pass at then-HEAD `da15b53bf452dc779fa451e268fc7a465e778a31` (extraction `/home/coding/scratch/hoop-53fc6b3b-head.dRXexo`, exit `0`) was recorded here before HEAD advanced; it is superseded by the committed-HEAD verification above.
