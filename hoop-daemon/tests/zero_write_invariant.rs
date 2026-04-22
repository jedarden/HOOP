//! Integration test for the zero-write invariant
//!
//! CI commands:
//!   cargo test -p hoop-daemon --features=zero-write-v01 --test zero_write_invariant
//!   cargo check -p hoop-daemon --features=zero-write-v01
//!   cargo check -p hoop --features=zero-write-v01
//!
//! This test verifies:
//! 1. Read verbs pass the runtime assertion
//! 2. Write verbs panic at runtime (belt-and-suspenders)
//! 3. The `invoke_br_write` function is not compiled under zero-write-v01
//! 4. The ZERO_WRITE_V01_ACTIVE const reflects the feature state
//! 5. The startup validation function runs cleanly

use hoop_daemon::br_verbs::{self, ReadVerb, WriteVerb, is_write_verb, assert_read_only};

// ---------------------------------------------------------------------------
// Classification tests
// ---------------------------------------------------------------------------

#[test]
fn test_read_verbs_are_not_write_verbs() {
    for verb in &["list", "get", "status", "--version", "doctor", "log", "show"] {
        assert!(!is_write_verb(verb), "read verb '{}' classified as write", verb);
    }
}

#[test]
fn test_write_verbs_are_detected() {
    for verb in &["create", "close", "update", "release", "claim", "depend"] {
        assert!(is_write_verb(verb), "write verb '{}' not classified as write", verb);
    }
}

// ---------------------------------------------------------------------------
// Runtime assertion tests
// ---------------------------------------------------------------------------

#[test]
fn test_assert_read_only_allows_read_verbs() {
    for verb in &["list", "get", "status", "--version", "doctor", "log", "show"] {
        assert_read_only(verb);
    }
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br create")]
fn test_create_panics() {
    assert_read_only("create");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br close")]
fn test_close_panics() {
    assert_read_only("close");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br update")]
fn test_update_panics() {
    assert_read_only("update");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br release")]
fn test_release_panics() {
    assert_read_only("release");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br claim")]
fn test_claim_panics() {
    assert_read_only("claim");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br depend")]
fn test_depend_panics() {
    assert_read_only("depend");
}

// ---------------------------------------------------------------------------
// Command builder tests
// ---------------------------------------------------------------------------

#[test]
fn test_invoke_br_read_produces_valid_command() {
    let cmd = br_verbs::invoke_br_read(ReadVerb::List, &["--json"]);
    assert_eq!(cmd.get_program(), "br");
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.contains(&std::ffi::OsStr::new("list")));
    assert!(args.contains(&std::ffi::OsStr::new("--json")));
}

#[test]
fn test_invoke_br_string_read_verb() {
    let cmd = br_verbs::invoke_br("get", &["bd-abc123"]);
    assert_eq!(cmd.get_program(), "br");
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.contains(&std::ffi::OsStr::new("get")));
    assert!(args.contains(&std::ffi::OsStr::new("bd-abc123")));
}

#[test]
#[should_panic(expected = "zero-write invariant violated")]
fn test_invoke_br_string_create_panics() {
    br_verbs::invoke_br("create", &["--json", "{}"]);
}

#[test]
#[should_panic(expected = "zero-write invariant violated")]
fn test_invoke_br_string_close_panics() {
    br_verbs::invoke_br("close", &["bd-abc123"]);
}

#[test]
#[should_panic(expected = "zero-write invariant violated")]
fn test_invoke_br_string_update_panics() {
    br_verbs::invoke_br("update", &["bd-abc123"]);
}

// ---------------------------------------------------------------------------
// Enum ↔ name consistency tests
// ---------------------------------------------------------------------------

#[test]
fn test_write_verb_enum_matches_names() {
    let verbs = [
        (WriteVerb::Create, "create"),
        (WriteVerb::Close, "close"),
        (WriteVerb::Update, "update"),
        (WriteVerb::Release, "release"),
        (WriteVerb::Claim, "claim"),
        (WriteVerb::Depend, "depend"),
    ];
    for (verb, name) in verbs {
        assert_eq!(verb.as_str(), name);
    }
}

#[test]
fn test_read_verb_enum_matches_names() {
    let verbs = [
        (ReadVerb::List, "list"),
        (ReadVerb::Get, "get"),
        (ReadVerb::Status, "status"),
        (ReadVerb::Version, "--version"),
        (ReadVerb::Doctor, "doctor"),
        (ReadVerb::Log, "log"),
        (ReadVerb::Show, "show"),
    ];
    for (verb, name) in verbs {
        assert_eq!(verb.as_str(), name);
    }
}

// ---------------------------------------------------------------------------
// Feature flag and startup validation tests
// ---------------------------------------------------------------------------

#[test]
fn test_zero_write_const_matches_feature() {
    assert_eq!(
        br_verbs::ZERO_WRITE_V01_ACTIVE,
        cfg!(feature = "zero-write-v01"),
        "ZERO_WRITE_V01_ACTIVE should match cfg!(feature = \"zero-write-v01\")"
    );
}

#[test]
fn test_validate_zero_write_invariant_runs_cleanly() {
    // Should never panic — just logs and verifies classification consistency
    br_verbs::validate_zero_write_invariant();
}

// ---------------------------------------------------------------------------
// Compile-time enforcement documentation
// ---------------------------------------------------------------------------
//
// When zero-write-v01 is active, invoke_br_write should not compile.
// The following would be a compile error under the feature:
//   br_verbs::invoke_br_write(WriteVerb::Create, &[]);
//
// CI enforcement:
//   1. `cargo check -p hoop-daemon --features=zero-write-v01` must succeed,
//      proving no code path references invoke_br_write when the feature is on.
//   2. `cargo check` (without feature) must also succeed, proving the
//      function exists for future phases.
//   3. `cargo test -p hoop-daemon --features=zero-write-v01 --test zero_write_invariant`
//      must pass all tests.
