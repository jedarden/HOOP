# bf-1sjxx: Fix hoop-daemon compile errors

## Task
Fix hoop-daemon compile errors: 95 errors → 0 (cargo check clean)

## Verification
Both acceptance criteria pass:

```bash
$ nix-shell -p pkg-config openssl --run "cargo check --package hoop-daemon 2>&1 | grep '^error\[ ' | wc -l"
0

$ nix-shell -p pkg-config openssl --run "cargo clippy --package hoop-daemon 2>&1 | grep '^error\[' | wc -l"
0
```

The package compiles cleanly with 0 errors. The 95 errors mentioned in the task description (ToSchema trait bounds, type mismatches, missing generics) have already been resolved in the codebase.

## Status
Complete - all acceptance criteria met.
