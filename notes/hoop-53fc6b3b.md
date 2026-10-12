# Targeted load-test integration compile

## Verification at committed HEAD (authoritative)

- HEAD: `ed1ccfcb232ce830a01c34668bee8274cce5dbee`
- Command: `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
- Exit code: `0`
- Extraction: clean `git archive HEAD` copy at `/home/coding/scratch/hoop-53fc6b3b-load.tzUlBd` (retained because the broader definition-of-done test step failed)
- Build path: `nix-shell --run` on NixOS with nix-provided cargo 1.98.1; the `~/.local/bin/cargo` wrapper did not run, so this was local execution rather than an iad-ci submit. Cargo used `/build/target`.
- Result: the `load_test_integration` test target compiles at current committed HEAD, including the post-`a4737ad6` changes.

Captured output tail:

```text
warning: `hoop-daemon` (test "load_test_integration") generated 6 warnings (run `cargo fix --test "load_test_integration" -p hoop-daemon` to apply 5 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 51s
  Executable tests/load_test_integration.rs (/build/target/debug/deps/load_test_integration-7a989f4ddf608c3a)
TARGETED_EXIT_CODE=0
```

## Definition-of-done check (same extraction)

- `cargo build --all-targets` → exit `0` (full log: `/home/coding/scratch/hoop-53fc6b3b-dod-current.log`).
- `cargo test` was run via `timeout --foreground 12m nix-shell --run 'cargo test'`; it reported unrelated existing unit-test failures and stalled integration tests, then was interrupted at exit `130` after the bound. The retained log is `/home/coding/scratch/hoop-53fc6b3b-test-current.log`; this is the pre-existing Phase 1 gate condition tracked on `hoop-7ae9fbff`, outside this bead's targeted compile-check scope.

## Prior evidence (superseded HEAD record)

Earlier passes at `7274c233e01281e5916dddcc5ecafd319bf4ca57` and `da15b53bf452dc779fa451e268fc7a465e778a31` were recorded here before HEAD advanced; they are superseded by the committed-HEAD verification above.
