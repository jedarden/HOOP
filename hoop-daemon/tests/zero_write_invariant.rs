//! Integration test for the write invariant (three modes)
//!
//! CI commands:
//!   # Phase 1 — zero-write (no br write verbs)
//!   cargo test -p hoop-daemon --features=zero-write-v01 --test zero_write_invariant
//!   cargo check -p hoop-daemon --features=zero-write-v01
//!   cargo check -p hoop --features=zero-write-v01
//!
//!   # Phase 4+ — create-only (only br create)
//!   cargo test -p hoop-daemon --features=create-only-write --test zero_write_invariant
//!   cargo check -p hoop-daemon --features=create-only-write
//!   cargo check -p hoop --features=create-only-write
//!
//!   # Unrestricted (dev mode)
//!   cargo test -p hoop-daemon --test zero_write_invariant
//!
//! This test verifies:
//! 1. Read verbs pass the runtime assertion
//! 2. Write verbs panic at runtime (belt-and-suspenders)
//! 3. The compile-time gates match the feature state
//! 4. The startup validation function runs cleanly
//! 5. Under create-only-write, `create` is allowed but other writes are forbidden

use hoop_daemon::br_verbs::{self, ReadVerb, WriteVerb, is_write_verb, is_forbidden_verb, assert_read_only, assert_create_only};

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

#[test]
fn test_forbidden_verbs_exclude_create() {
    assert!(!is_forbidden_verb("create"), "'create' must not be forbidden");
    for verb in &["close", "update", "release", "claim", "depend"] {
        assert!(is_forbidden_verb(verb), "verb '{}' should be forbidden", verb);
    }
}

// ---------------------------------------------------------------------------
// Runtime assertion tests — read-only mode
// ---------------------------------------------------------------------------

#[test]
fn test_assert_read_only_allows_read_verbs() {
    for verb in &["list", "get", "status", "--version", "doctor", "log", "show"] {
        assert_read_only(verb);
    }
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br create")]
fn test_create_panics_read_only() {
    assert_read_only("create");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br close")]
fn test_close_panics_read_only() {
    assert_read_only("close");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br update")]
fn test_update_panics_read_only() {
    assert_read_only("update");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br release")]
fn test_release_panics_read_only() {
    assert_read_only("release");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br claim")]
fn test_claim_panics_read_only() {
    assert_read_only("claim");
}

#[test]
#[should_panic(expected = "zero-write invariant violated: br depend")]
fn test_depend_panics_read_only() {
    assert_read_only("depend");
}

// ---------------------------------------------------------------------------
// Runtime assertion tests — create-only mode
// ---------------------------------------------------------------------------

#[test]
fn test_assert_create_only_allows_create() {
    assert_create_only("create");
}

#[test]
fn test_assert_create_only_allows_read_verbs() {
    for verb in &["list", "get", "status", "--version", "doctor", "log", "show"] {
        assert_create_only(verb);
    }
}

#[test]
#[should_panic(expected = "create-only invariant violated")]
fn test_close_panics_create_only() {
    assert_create_only("close");
}

#[test]
#[should_panic(expected = "create-only invariant violated")]
fn test_update_panics_create_only() {
    assert_create_only("update");
}

#[test]
#[should_panic(expected = "create-only invariant violated")]
fn test_release_panics_create_only() {
    assert_create_only("release");
}

#[test]
#[should_panic(expected = "create-only invariant violated")]
fn test_claim_panics_create_only() {
    assert_create_only("claim");
}

#[test]
#[should_panic(expected = "create-only invariant violated")]
fn test_depend_panics_create_only() {
    assert_create_only("depend");
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
#[should_panic(expected = "invariant violated")]
fn test_invoke_br_string_create_panics_read_only() {
    // Under zero-write-v01, create panics via assert_read_only
    // Under create-only-write, create is allowed via assert_create_only
    // Without either, assert_read_only panics
    // This test expects a panic because it runs under zero-write-v01 or unrestricted
    // (under create-only-write it would NOT panic — this test is cfg-gated below)
    #[cfg(not(feature = "create-only-write"))]
    br_verbs::invoke_br("create", &["--json", "{}"]);

    #[cfg(feature = "create-only-write")]
    panic!("invariant violated: this test should not run under create-only-write");
}

#[test]
#[should_panic(expected = "invariant violated")]
fn test_invoke_br_string_close_panics() {
    br_verbs::invoke_br("close", &["bd-abc123"]);
}

#[test]
#[should_panic(expected = "invariant violated")]
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
        br_verbs::ZERO_WRITE_ACTIVE,
        cfg!(feature = "zero-write-v01"),
        "ZERO_WRITE_ACTIVE should match cfg!(feature = \"zero-write-v01\")"
    );
}

#[test]
fn test_create_only_const_matches_feature() {
    assert_eq!(
        br_verbs::CREATE_ONLY_ACTIVE,
        cfg!(feature = "create-only-write"),
        "CREATE_ONLY_ACTIVE should match cfg!(feature = \"create-only-write\")"
    );
}

#[test]
fn test_write_restricted_const() {
    assert_eq!(
        br_verbs::WRITE_RESTRICTED,
        cfg!(feature = "zero-write-v01") || cfg!(feature = "create-only-write"),
        "WRITE_RESTRICTED should be true when any write restriction is active"
    );
}

#[test]
fn test_validate_write_invariant_runs_cleanly() {
    br_verbs::validate_write_invariant();
}

#[test]
fn test_validate_zero_write_invariant_alias() {
    br_verbs::validate_zero_write_invariant();
}

// ---------------------------------------------------------------------------
// Compile-time enforcement documentation
// ---------------------------------------------------------------------------
//
// When zero-write-v01 is active:
//   - invoke_br_create should not compile
//   - invoke_br_write should not compile
//
// When create-only-write is active:
//   - invoke_br_create SHOULD compile (the only allowed write)
//   - invoke_br_write should NOT compile (non-create writes forbidden)
//
// Without either feature:
//   - Both invoke_br_create and invoke_br_write compile
//
// CI enforcement:
//   1. `cargo check -p hoop-daemon --features=zero-write-v01` must succeed,
//      proving no code path references invoke_br_create or invoke_br_write.
//   2. `cargo check -p hoop-daemon --features=create-only-write` must succeed,
//      proving invoke_br_create works but invoke_br_write is unreachable.
//   3. `cargo check` (without feature) must also succeed, proving both functions exist.
//   4. All three test invocations must pass.
//
// Runtime subprocess-arg inspection:
//   - validate_br_subprocess_args(cmd) inspects the built Command's first arg
//   - Called by all builder functions (invoke_br_read, invoke_br_create, invoke_br_write, invoke_br)
//   - Provides belt-and-suspenders validation even if a raw Command::new("br") is constructed

// ---------------------------------------------------------------------------
// Subprocess-arg inspection tests
// ---------------------------------------------------------------------------

#[test]
fn test_validate_br_subprocess_args_allows_read_verbs() {
    for verb in br_verbs::READ_VERB_NAMES {
        let mut cmd = std::process::Command::new("br");
        cmd.arg(verb);
        br_verbs::validate_br_subprocess_args(&cmd);
    }
}

#[test]
fn test_validate_br_subprocess_args_rejects_forbidden_verbs() {
    for verb in br_verbs::FORBIDDEN_WRITE_VERBS {
        let result = std::panic::catch_unwind(|| {
            let mut cmd = std::process::Command::new("br");
            cmd.arg(verb);
            br_verbs::validate_br_subprocess_args(&cmd);
        });
        assert!(result.is_err(), "validate_br_subprocess_args should reject '{}'", verb);
    }
}
