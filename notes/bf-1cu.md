# bf-1cu: Build and install hoop binary after Phase 1 CI gate passes

## Status: PRECONDITION NOT MET

This bead could not be completed because its prerequisite condition has not been met.

## Prerequisite Check

Bead bf-5mpcl (Phase 1 CI gate) is still **OPEN** with status:
- `cargo test` - NOT PASSED
- `cargo clippy -- -D warnings` - NOT PASSED  
- `hoop status --json | jq .` - NOT VERIFIED

## Verification

Ran `cargo check` on 2026-06-28:
```
error: could not compile `hoop-daemon` (lib) due to 54 previous errors; 74 warnings emitted
```

The codebase does not compile. Per AGENTS.md: "ACTUAL STATE (as of 2026-06-28): Phase 0 complete. Phase 1 in progress. `cargo build` FAILS (36 compilation errors)."

## Why This Bead Can't Proceed

The bead description states:
> "Once bf-5mpcl (Phase 1 CI gate) is green, build the release binary..."

This is a conditional dependency. The condition is not met.

## Next Steps

1. Complete Phase 1 compilation fixes (bead bf-1sjxx)
2. Pass Phase 1 CI gate (bead bf-5mpcl)
3. THEN retry this bead (bf-1cu)

This bead must remain open until the CI gate passes.

## Date: 2026-06-28
