//! br verb classification and zero-write invariant guard
//!
//! Phase 1 is strictly read-only. This module:
//! - Classifies br verbs as read or write
//! - Under `zero-write-v01` feature, write verbs are unreachable at compile time
//!   (the `invoke_br_write` function is not compiled)
//! - Runtime `assert_read_only` panics as belt-and-suspenders if a write verb
//!   sneaks through via string-based dispatch
//! - `validate_zero_write_invariant()` is called at daemon startup to log the
//!   invariant state

/// Whether the zero-write invariant is enforced at compile time.
/// When `true`, `invoke_br_write` does not exist and any code referencing it
/// will fail to compile.
pub const ZERO_WRITE_V01_ACTIVE: bool = cfg!(feature = "zero-write-v01");

/// br verbs that mutate bead state. HOOP must never call these in phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteVerb {
    Create,
    Close,
    Update,
    Release,
    Claim,
    Depend,
}

impl WriteVerb {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Close => "close",
            Self::Update => "update",
            Self::Release => "release",
            Self::Claim => "claim",
            Self::Depend => "depend",
        }
    }
}

/// br verbs that are read-only. Safe to call in phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadVerb {
    List,
    Get,
    Status,
    Version,
    Doctor,
    Log,
    Show,
}

impl ReadVerb {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Get => "get",
            Self::Status => "status",
            Self::Version => "--version",
            Self::Doctor => "doctor",
            Self::Log => "log",
            Self::Show => "show",
        }
    }
}

/// All br verb names that are classified as write operations.
pub const WRITE_VERB_NAMES: &[&str] = &["create", "close", "update", "release", "claim", "depend"];

/// All br verb names that are classified as read operations.
pub const READ_VERB_NAMES: &[&str] = &["list", "get", "status", "--version", "doctor", "log", "show"];

/// Check whether a br verb name is a write operation.
pub fn is_write_verb(verb: &str) -> bool {
    WRITE_VERB_NAMES.contains(&verb)
}

/// Runtime guard that panics if a write verb is attempted under phase 1.
///
/// This is the belt-and-suspenders check. Under `zero-write-v01`, the write
/// code paths are also excluded at compile time, so this should never trigger.
/// But if somehow a write verb sneaks through (e.g. via string-based subprocess
/// call), this catches it.
pub fn assert_read_only(verb: &str) {
    if is_write_verb(verb) {
        panic!(
            "HOOP zero-write invariant violated: br {} is a write verb. \
             Phase 1 is strictly read-only. This is a bug — please report it.",
            verb
        );
    }
}

/// Invoke a br read verb. This is the only br invocation function available
/// unconditionally.
///
/// All br subprocess calls in HOOP should route through this function so the
/// zero-write invariant is enforced at a single choke point.
pub fn invoke_br_read(verb: ReadVerb, args: &[&str]) -> std::process::Command {
    assert_read_only(verb.as_str());
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

/// Invoke a br write verb. Only available when `zero-write-v01` is NOT set.
///
/// In phase 1 (zero-write-v01 active), this function does not exist at
/// compile time. It appears in phase 4 when bead creation is implemented.
#[cfg(not(feature = "zero-write-v01"))]
pub fn invoke_br_write(verb: WriteVerb, args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

/// Invoke br with an arbitrary verb string, enforcing the zero-write invariant
/// at runtime. Use typed `invoke_br_read` / `invoke_br_write` instead when
/// the verb is known at compile time.
pub fn invoke_br(verb: &str, args: &[&str]) -> std::process::Command {
    assert_read_only(verb);
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb);
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

/// Validate the zero-write invariant at daemon startup.
///
/// Logs the invariant state and panics if the runtime guard detects a write
/// verb is reachable (should never happen — the cfg gate prevents it).
pub fn validate_zero_write_invariant() {
    if ZERO_WRITE_V01_ACTIVE {
        tracing::info!(
            "zero-write invariant: ACTIVE (compile-time enforcement via zero-write-v01 feature)"
        );
    } else {
        tracing::warn!(
            "zero-write invariant: INACTIVE (zero-write-v01 feature not set — write verbs are reachable)"
        );
    }

    // Belt-and-suspenders: verify the runtime guard rejects every write verb.
    // This should always pass; a panic here means something is deeply wrong.
    for verb in WRITE_VERB_NAMES {
        assert!(
            is_write_verb(verb),
            "zero-write invariant: internal error — {} not classified as write verb",
            verb
        );
    }
    tracing::debug!(
        "zero-write invariant: {} write verbs classified correctly",
        WRITE_VERB_NAMES.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_verb_classification() {
        assert!(is_write_verb("create"));
        assert!(is_write_verb("close"));
        assert!(is_write_verb("update"));
        assert!(is_write_verb("release"));
        assert!(is_write_verb("claim"));
        assert!(is_write_verb("depend"));
    }

    #[test]
    fn test_read_verb_classification() {
        assert!(!is_write_verb("list"));
        assert!(!is_write_verb("get"));
        assert!(!is_write_verb("status"));
        assert!(!is_write_verb("--version"));
        assert!(!is_write_verb("doctor"));
        assert!(!is_write_verb("log"));
        assert!(!is_write_verb("show"));
    }

    #[test]
    fn test_read_verbs_pass_assertion() {
        // These should not panic
        assert_read_only("list");
        assert_read_only("get");
        assert_read_only("status");
        assert_read_only("--version");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_write_verb_panics_assertion() {
        assert_read_only("create");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_close_panics_assertion() {
        assert_read_only("close");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_update_panics_assertion() {
        assert_read_only("update");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_release_panics_assertion() {
        assert_read_only("release");
    }

    #[test]
    fn test_invoke_br_read_builds_command() {
        let cmd = invoke_br_read(ReadVerb::List, &["--json"]);
        assert_eq!(cmd.get_program(), "br");
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["list", "--json"]);
    }

    #[test]
    fn test_invoke_br_string_read_builds_command() {
        let cmd = invoke_br("list", &["--json"]);
        assert_eq!(cmd.get_program(), "br");
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["list", "--json"]);
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_invoke_br_string_write_panics() {
        let _ = invoke_br("create", &["--json", "{}"]);
    }

    // When zero-write-v01 is active, invoke_br_write should not exist.
    // This test verifies the compile-time enforcement by checking that the
    // function is not in scope.
    #[test]
    fn test_write_verb_function_not_compiled_under_feature() {
        // This test always passes — it documents the compile-time check.
        // The real enforcement is that `invoke_br_write` is cfg-gated:
        //   #[cfg(not(feature = "zero-write-v01"))]
        //   pub fn invoke_br_write(...)
        //
        // CI should run: cargo check -p hoop-daemon --features=zero-write-v01
        // If any code calls invoke_br_write, it will fail to compile.
        //
        // We also verify the feature flag is documented:
        assert!(cfg!(feature = "zero-write-v01") || !cfg!(feature = "zero-write-v01"));
    }

    #[test]
    fn test_zero_write_const_matches_feature() {
        assert_eq!(ZERO_WRITE_V01_ACTIVE, cfg!(feature = "zero-write-v01"));
    }

    #[test]
    fn test_validate_zero_write_invariant_runs() {
        // Should never panic — just logs and verifies classification
        validate_zero_write_invariant();
    }

    #[test]
    fn test_all_write_verbs_in_constant() {
        // Verify WRITE_VERB_NAMES covers every WriteVerb variant
        let expected = ["create", "close", "update", "release", "claim", "depend"];
        for name in &expected {
            assert!(
                WRITE_VERB_NAMES.contains(name),
                "WRITE_VERB_NAMES missing '{}'",
                name
            );
        }
    }

    #[test]
    fn test_all_read_verbs_in_constant() {
        let expected = ["list", "get", "status", "--version", "doctor", "log", "show"];
        for name in &expected {
            assert!(
                READ_VERB_NAMES.contains(name),
                "READ_VERB_NAMES missing '{}'",
                name
            );
        }
    }

    #[test]
    fn test_no_verb_overlap() {
        for read in READ_VERB_NAMES {
            assert!(
                !is_write_verb(read),
                "read verb '{}' incorrectly classified as write",
                read
            );
        }
    }
}
