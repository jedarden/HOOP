//! Compile-fail tests: verify invoke_br_write cannot be called under create-only-write.
//!
//! These tests only run with `--features=create-only-write`. They use trybuild to
//! verify that files in `tests/ui/` that call `invoke_br_write` fail to compile.
//!
//! CI command:
//!   cargo test -p hoop-daemon --features=create-only-write --test compile_fail_create_only

#[cfg(feature = "create-only-write")]
#[test]
fn invoke_br_write_is_not_compilable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invoke_br_write_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_close_raw_forbidden.rs");
    t.compile_fail("tests/ui/invoke_br_update_forbidden.rs");
}

#[cfg(not(feature = "create-only-write"))]
#[test]
fn invoke_br_write_compile_fail_skipped_without_feature() {
    eprintln!(
        "compile-fail test skipped: invoke_br_write exists when create-only-write is not active"
    );
}
