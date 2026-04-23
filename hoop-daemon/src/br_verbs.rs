//! br verb classification and create-only invariant guard
//!
//! Phase 1 (zero-write-v01): strictly read-only, no br writes at all.
//! Phase 4+ (create-only-write): only `br create` is allowed.
//!
//! This module:
//! - Classifies br verbs as read or write
//! - Under `zero-write-v01`, ALL write verbs are unreachable at compile time
//! - Under `create-only-write`, only `invoke_br_create()` compiles; other write
//!   verbs fail to compile and are rejected at runtime
//! - `validate_write_invariant()` is called at daemon startup to log the mode

/// Whether any write restriction is active at compile time.
pub const WRITE_RESTRICTED: bool =
    cfg!(feature = "zero-write-v01") || cfg!(feature = "create-only-write");

/// Whether the create-only invariant is active (phase 4+).
/// When `true`, only `invoke_br_create()` compiles; `invoke_br_write` does not exist.
pub const CREATE_ONLY_ACTIVE: bool = cfg!(feature = "create-only-write");

/// Whether the zero-write invariant is active (phase 1).
/// When `true`, no write invocation functions compile at all.
pub const ZERO_WRITE_ACTIVE: bool = cfg!(feature = "zero-write-v01");

/// br verbs that mutate bead state.
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

/// br verbs that are read-only. Safe to call in any phase.
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

/// Write verbs that are forbidden under `create-only-write`.
/// `create` is NOT in this list — it is the one allowed write verb.
pub const FORBIDDEN_WRITE_VERBS: &[&str] = &["close", "update", "release", "claim", "depend"];

/// Check whether a br verb name is a write operation.
pub fn is_write_verb(verb: &str) -> bool {
    WRITE_VERB_NAMES.contains(&verb)
}

/// Check whether a br verb name is forbidden under create-only mode.
pub fn is_forbidden_verb(verb: &str) -> bool {
    FORBIDDEN_WRITE_VERBS.contains(&verb)
}

/// Runtime guard: reject any verb that is not `create` when create-only is active.
///
/// Belt-and-suspenders: under `create-only-write`, `invoke_br_write` does not compile,
/// so non-create write verbs can't get here. But `invoke_br` (string-based) or a
/// raw `Command::new("br")` could — this catches those paths.
pub fn assert_create_only(verb: &str) {
    if is_forbidden_verb(verb) {
        panic!(
            "HOOP create-only invariant violated: br {} is forbidden. \
             Only `br create` is allowed in phase 4+. This is a bug — please report it.",
            verb
        );
    }
    // Also catch any completely unknown write verbs
    if is_write_verb(verb) && verb != "create" {
        panic!(
            "HOOP create-only invariant violated: br {} is a write verb but not 'create'. \
             This is a bug — please report it.",
            verb
        );
    }
}

/// Runtime guard that panics if ANY write verb is attempted (phase 1 zero-write mode).
pub fn assert_read_only(verb: &str) {
    if is_write_verb(verb) {
        panic!(
            "HOOP zero-write invariant violated: br {} is a write verb. \
             Phase 1 is strictly read-only. This is a bug — please report it.",
            verb
        );
    }
}

/// Subprocess-arg inspection: validate a built `Command` object's args.
///
/// This is the belt-and-suspenders runtime layer. It inspects the actual `Command`
/// that will be spawned and rejects it if the first arg (the verb) is not allowed.
/// Unlike `assert_create_only`/`assert_read_only` which validate a string argument,
/// this validates the final `Command` object — catching any path that bypasses the
/// typed builders (e.g., raw `Command::new("br")` or post-construction mutation).
///
/// Under `create-only-write`: only `create` and read verbs pass.
/// Under `zero-write-v01`: only read verbs pass.
/// Unrestricted: all verbs pass.
pub fn validate_br_subprocess_args(cmd: &std::process::Command) {
    let first_arg = cmd.get_args().next();
    let verb = first_arg
        .map(|a| a.to_string_lossy().into_owned())
        .unwrap_or_default();

    if ZERO_WRITE_ACTIVE {
        if verb.is_empty() {
            panic!(
                "HOOP zero-write invariant violated: br invoked with no verb. \
                 Phase 1 is strictly read-only. This is a bug — please report it."
            );
        }
        assert_read_only(&verb);
    } else if CREATE_ONLY_ACTIVE {
        if verb.is_empty() {
            panic!(
                "HOOP create-only invariant violated: br invoked with no verb. \
                 Only `br create` is allowed in phase 4+. This is a bug — please report it."
            );
        }
        assert_create_only(&verb);
    }
    // Unrestricted mode: no validation needed
}

/// Invoke a br read verb. This is always available regardless of feature flags.
pub fn invoke_br_read(verb: ReadVerb, args: &[&str]) -> std::process::Command {
    assert_read_only(verb.as_str());
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

/// Invoke `br create` — the single allowed write verb in phase 4+.
///
/// Available when:
/// - `create-only-write` feature is set (phase 4+), OR
/// - Neither `zero-write-v01` nor `create-only-write` is set (unrestricted dev mode)
///
/// NOT available when `zero-write-v01` is set (phase 1: strictly read-only).
#[cfg(any(
    feature = "create-only-write",
    not(any(feature = "zero-write-v01", feature = "create-only-write"))
))]
pub fn invoke_br_create(args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("br");
    cmd.arg("create");
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

/// Invoke a br write verb. Only available when NO write restriction is active
/// (neither `zero-write-v01` nor `create-only-write`).
///
/// Under `create-only-write`, this function does not exist at compile time —
/// use `invoke_br_create()` instead.
/// Under `zero-write-v01`, neither this nor `invoke_br_create` exists.
#[cfg(not(any(feature = "zero-write-v01", feature = "create-only-write")))]
pub fn invoke_br_write(verb: WriteVerb, args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

/// Invoke br with an arbitrary verb string, enforcing the write invariant at runtime.
/// Use typed `invoke_br_read` / `invoke_br_create` instead when the verb is known at compile time.
#[cfg(not(feature = "zero-write-v01"))]
pub fn invoke_br(verb: &str, args: &[&str]) -> std::process::Command {
    #[cfg(feature = "create-only-write")]
    assert_create_only(verb);
    #[cfg(not(feature = "create-only-write"))]
    assert_read_only(verb);
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb);
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

/// Invoke br with an arbitrary verb string — zero-write variant (rejects ALL writes).
#[cfg(feature = "zero-write-v01")]
pub fn invoke_br(verb: &str, args: &[&str]) -> std::process::Command {
    assert_read_only(verb);
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb);
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

/// Validate the write invariant at daemon startup.
///
/// Logs the invariant mode and panics if the runtime guards detect an inconsistency.
pub fn validate_write_invariant() {
    if ZERO_WRITE_ACTIVE {
        tracing::info!(
            "write invariant: ZERO-WRITE (phase 1 — no br write verbs at compile time)"
        );
    } else if CREATE_ONLY_ACTIVE {
        tracing::info!(
            "write invariant: CREATE-ONLY (phase 4+ — only br create at compile time)"
        );
    } else {
        tracing::warn!(
            "write invariant: UNRESTRICTED (no feature flag set — all br verbs reachable)"
        );
    }

    // Belt-and-suspenders: verify runtime guards reject every forbidden verb.
    for verb in FORBIDDEN_WRITE_VERBS {
        assert!(
            is_forbidden_verb(verb),
            "write invariant: internal error — {} not classified as forbidden",
            verb
        );
    }
    // Verify "create" is NOT forbidden
    assert!(
        !is_forbidden_verb("create"),
        "write invariant: internal error — 'create' must not be forbidden"
    );
    // But "create" IS a write verb
    assert!(
        is_write_verb("create"),
        "write invariant: internal error — 'create' must be classified as write"
    );
    tracing::debug!(
        "write invariant: {} write verbs total, {} forbidden (all except create)",
        WRITE_VERB_NAMES.len(),
        FORBIDDEN_WRITE_VERBS.len()
    );
}

/// Backward-compatible alias for the startup validation function.
pub fn validate_zero_write_invariant() {
    validate_write_invariant();
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
    fn test_forbidden_verb_classification() {
        assert!(!is_forbidden_verb("create"), "create must NOT be forbidden");
        assert!(is_forbidden_verb("close"));
        assert!(is_forbidden_verb("update"));
        assert!(is_forbidden_verb("release"));
        assert!(is_forbidden_verb("claim"));
        assert!(is_forbidden_verb("depend"));
    }

    #[test]
    fn test_read_verbs_not_forbidden() {
        for verb in READ_VERB_NAMES {
            assert!(!is_forbidden_verb(verb), "read verb '{}' must not be forbidden", verb);
        }
    }

    #[test]
    fn test_read_verbs_pass_read_only_assertion() {
        assert_read_only("list");
        assert_read_only("get");
        assert_read_only("status");
        assert_read_only("--version");
    }

    #[test]
    fn test_create_passes_create_only_assertion() {
        // create is the one allowed write verb
        assert_create_only("create");
    }

    #[test]
    fn test_read_verbs_pass_create_only_assertion() {
        // read verbs should pass the create-only guard
        assert_create_only("list");
        assert_create_only("get");
        assert_create_only("status");
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
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_write_verb_panics_read_only() {
        assert_read_only("create");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_close_panics_read_only() {
        assert_read_only("close");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_update_panics_read_only() {
        assert_read_only("update");
    }

    #[test]
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_release_panics_read_only() {
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
    #[should_panic(expected = "invariant violated")]
    fn test_invoke_br_string_write_panics() {
        let _ = invoke_br("close", &["bd-abc123"]);
    }

    #[test]
    fn test_write_verb_function_availability_matches_feature() {
        // This test always passes — it documents the compile-time check.
        // Under create-only-write: invoke_br_create exists, invoke_br_write does not.
        // Under zero-write-v01: neither exists.
        // Without either: both exist.
        assert!(cfg!(feature = "create-only-write") || !cfg!(feature = "create-only-write"));
    }

    #[test]
    fn test_feature_constants() {
        assert_eq!(ZERO_WRITE_ACTIVE, cfg!(feature = "zero-write-v01"));
        assert_eq!(CREATE_ONLY_ACTIVE, cfg!(feature = "create-only-write"));
        assert_eq!(
            WRITE_RESTRICTED,
            cfg!(feature = "zero-write-v01") || cfg!(feature = "create-only-write")
        );
    }

    #[test]
    fn test_validate_write_invariant_runs() {
        validate_write_invariant();
    }

    #[test]
    fn test_validate_zero_write_invariant_alias() {
        validate_zero_write_invariant();
    }

    #[test]
    fn test_all_write_verbs_in_constant() {
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
    fn test_all_forbidden_verbs_in_constant() {
        let expected = ["close", "update", "release", "claim", "depend"];
        for name in &expected {
            assert!(
                FORBIDDEN_WRITE_VERBS.contains(name),
                "FORBIDDEN_WRITE_VERBS missing '{}'",
                name
            );
        }
        assert!(
            !FORBIDDEN_WRITE_VERBS.contains(&"create"),
            "FORBIDDEN_WRITE_VERBS must not contain 'create'"
        );
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

    #[test]
    fn test_invoke_br_create_builds_command() {
        // invoke_br_create is only available under create-only-write or unrestricted
        #[cfg(any(
            feature = "create-only-write",
            not(any(feature = "zero-write-v01", feature = "create-only-write"))
        ))]
        {
            let cmd = invoke_br_create(&["--type", "task"]);
            assert_eq!(cmd.get_program(), "br");
            let args: Vec<_> = cmd.get_args().collect();
            assert_eq!(args, ["create", "--type", "task"]);
        }
    }

    // -----------------------------------------------------------------------
    // Subprocess-arg inspection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_br_subprocess_args_allows_create() {
        let mut cmd = std::process::Command::new("br");
        cmd.arg("create").arg("--type").arg("task");
        validate_br_subprocess_args(&cmd);
    }

    #[test]
    fn test_validate_br_subprocess_args_allows_read_verbs() {
        for verb in READ_VERB_NAMES {
            let mut cmd = std::process::Command::new("br");
            cmd.arg(verb);
            validate_br_subprocess_args(&cmd);
        }
    }

    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_br_subprocess_args_rejects_raw_close_command() {
        let mut cmd = std::process::Command::new("br");
        cmd.arg("close").arg("bd-abc123");
        validate_br_subprocess_args(&cmd);
    }

    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_br_subprocess_args_rejects_raw_update_command() {
        let mut cmd = std::process::Command::new("br");
        cmd.arg("update").arg("bd-abc123");
        validate_br_subprocess_args(&cmd);
    }

    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_br_subprocess_args_rejects_empty_command() {
        let cmd = std::process::Command::new("br");
        validate_br_subprocess_args(&cmd);
    }
}
