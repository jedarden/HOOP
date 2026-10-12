# Targeted load-test integration compile

## Verification at committed HEAD

- HEAD: `5a51929f51e9e69f1f10fee665c921f1caf7b3c7`
- Command: `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
- Exit code: `0`
- Extraction: clean `git archive HEAD` copy at `/home/coding/scratch/hoop-53fc6b3b-current.t20ane` (removed after the successful check)
- Build path: `nix-shell --run` on NixOS with nix-provided Cargo 1.98.1; the `~/.local/bin/cargo` wrapper did not run, so this was local execution rather than an iad-ci submit. Cargo used `/build/target`.
- Result: the `load_test_integration` test target compiles at the current committed HEAD.

Captured output tail:

```text
warning: `hoop-daemon` (test "load_test_integration") generated 6 warnings (run `cargo fix --test "load_test_integration" -p hoop-daemon` to apply 5 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.70s
  Executable tests/load_test_integration.rs (/build/target/debug/deps/load_test_integration-7a989f4ddf608c3a)
TARGETED_EXIT_CODE=0
```

Full captured output was retained at `/home/coding/scratch/hoop-53fc6b3b-current-load-test.log` during verification.
