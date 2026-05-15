# BF-3a9f4: Create-only invariant verification

## Finding

The trybuild tests in `compile_fail_create_only.rs` are **PASSING**, not failing. The create-only invariant is correctly enforced at compile time.

## Verification

Ran both test suites with `--features=create-only-write`:

```bash
# hoop-daemon
cargo test -p hoop-daemon --features=create-only-write --test compile_fail_create_only
# Result: test result: ok. 1 passed; 0 failed

# hoop-mcp
cargo test -p hoop-mcp --features=create-only-write --test compile_fail_create_only
# Result: test result: ok. 1 passed; 0 failed
```

All UI test fixtures correctly fail to compile with the expected error:
```
error[E0432]: unresolved import `hoop_daemon::br_verbs::invoke_br_write`
  |
4 | use hoop_daemon::br_verbs::{invoke_br_write, WriteVerb};
  |                             ^^^^^^^^^^^^^^^
  |                             no `invoke_br_write` in `br_verbs`
  |
note: found an item that was configured out
  --> src/br_verbs.rs
   |
   | #[cfg(not(any(feature = "zero-write-v01", feature = "create-only-write")))]
   |          ---------------------------------------------------------------- the item is gated here
```

## Implementation

The conditional compilation in both `hoop-daemon/src/br_verbs.rs` and `hoop-mcp/src/br_verbs.rs` is correct:

```rust
#[cfg(not(any(feature = "zero-write-v01", feature = "create-only-write")))]
pub fn invoke_br_write(verb: WriteVerb, args: &[&str]) -> std::process::Command {
    // ...
}
```

This ensures:
- Under `create-only-write`: `invoke_br_write` does NOT exist (only `invoke_br_create` exists)
- Under `zero-write-v01`: Neither `invoke_br_write` nor `invoke_br_create` exists
- Unrestricted: Both functions exist

## History

This was fixed in commit `0968d6a` on 2026-04-23:
"Zero-write br invariant: compile-time + runtime enforcement for create-only (phase 4+)"

## Conclusion

The invariant is properly enforced. The task description was based on outdated information.
