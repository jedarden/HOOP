//! Compile-fail tests: enforce the create-only invariant for br verbs.
//!
//! # The invariant (plan.md §3 principle 8, §6 Phase 1 deliverable 7)
//!
//! HOOP's ONLY write action is `br create`. All other br write verbs (close, update,
//! release, claim, depend) MUST be unreachable at compile time when the
//! `create-only-write` feature is active. This is enforced at TWO layers:
//!
//! 1. **Compile-time**: `invoke_br_write` does NOT exist under `create-only-write`.
//!    Only `invoke_br_create` compiles. This trybuild suite proves it.
//!
//! 2. **Runtime**: Even if someone bypasses the typed API (e.g., raw `Command::new("br")`),
//!    `validate_br_subprocess_args` in br_verbs.rs panics before the subprocess spawns.
//!
//! # How this works
//!
//! These tests only run with `--features=create-only-write`. They use trybuild to
//! verify that each file in `tests/ui/` that attempts to call `invoke_br_write` with
//! a forbidden verb FAILS to compile. If any fixture compiles successfully, the
//! invariant is broken and CI fails.
//!
//! # WARNING: Do not weaken this test
//!
//! - Each fixture corresponds to ONE forbidden verb. Removing or commenting out
//!   any fixture weakens the invariant and violates §3 principle 8.
//!
//! - If you add a new br write verb to br_verbs.rs, you MUST add a corresponding
//!   fixture here AND update FORBIDDEN_WRITE_VERBS.
//!
//! - If you believe this test is too strict, you are misunderstanding HOOP's
//!   architecture. HOOP NEVER mutates beads; only `br create` is allowed. All other
//!   mutations MUST go through `br` directly or NEEDLE workers.
//!
//! CI command:
//!   cargo test -p hoop-daemon --features=create-only-write --test compile_fail_create_only

#[cfg(feature = "create-only-write")]
#[test]
fn invoke_br_write_is_not_compilable() {
    let t = trybuild::TestCases::new();
    // All write verbs except create must fail to compile
    t.compile_fail("tests/ui/invoke_br_close_raw_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_claim_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_depend_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_release_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_update_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_write_forbidden.rs");
}

#[cfg(not(feature = "create-only-write"))]
#[test]
fn invoke_br_write_compile_fail_skipped_without_feature() {
    eprintln!(
        "compile-fail test skipped: invoke_br_write exists when create-only-write is not active"
    );
}
