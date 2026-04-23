//! br verb classification and create-only invariant guard
//!
//! Mirrors `hoop-daemon::br_verbs` — ensures the MCP server respects the same
//! write invariant: under `create-only-write`, only `br create` compiles.
//!
//! This module is the MCP crate's belt-and-suspenders enforcement:
//! - Compile-time: `invoke_br_write` does not exist under `create-only-write`
//! - Runtime: `validate_br_subprocess_args` inspects built `Command` objects
//! - Startup: `validate_write_invariant` logs the active mode

#[allow(dead_code)]
pub const WRITE_RESTRICTED: bool =
    cfg!(feature = "zero-write-v01") || cfg!(feature = "create-only-write");

pub const CREATE_ONLY_ACTIVE: bool = cfg!(feature = "create-only-write");

pub const ZERO_WRITE_ACTIVE: bool = cfg!(feature = "zero-write-v01");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WriteVerb {
    Create,
    Close,
    Update,
    Release,
    Claim,
    Depend,
}

impl WriteVerb {
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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

#[allow(dead_code)]
pub const WRITE_VERB_NAMES: &[&str] =
    &["create", "close", "update", "release", "claim", "depend"];

#[allow(dead_code)]
pub const READ_VERB_NAMES: &[&str] =
    &["list", "get", "status", "--version", "doctor", "log", "show"];

pub const FORBIDDEN_WRITE_VERBS: &[&str] = &["close", "update", "release", "claim", "depend"];

pub fn is_write_verb(verb: &str) -> bool {
    WRITE_VERB_NAMES.contains(&verb)
}

pub fn is_forbidden_verb(verb: &str) -> bool {
    FORBIDDEN_WRITE_VERBS.contains(&verb)
}

pub fn assert_create_only(verb: &str) {
    if is_forbidden_verb(verb) {
        panic!(
            "HOOP create-only invariant violated: br {} is forbidden. \
             Only `br create` is allowed in phase 4+. This is a bug — please report it.",
            verb
        );
    }
    if is_write_verb(verb) && verb != "create" {
        panic!(
            "HOOP create-only invariant violated: br {} is a write verb but not 'create'. \
             This is a bug — please report it.",
            verb
        );
    }
}

pub fn assert_read_only(verb: &str) {
    if is_write_verb(verb) {
        panic!(
            "HOOP zero-write invariant violated: br {} is a write verb. \
             Phase 1 is strictly read-only. This is a bug — please report it.",
            verb
        );
    }
}

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
}

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

#[cfg(not(any(feature = "zero-write-v01", feature = "create-only-write")))]
#[allow(dead_code)]
pub fn invoke_br_write(verb: WriteVerb, args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("br");
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    validate_br_subprocess_args(&cmd);
    cmd
}

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

    for verb in FORBIDDEN_WRITE_VERBS {
        assert!(
            is_forbidden_verb(verb),
            "write invariant: internal error — {} not classified as forbidden",
            verb
        );
    }
    assert!(
        !is_forbidden_verb("create"),
        "write invariant: internal error — 'create' must not be forbidden"
    );
    assert!(
        is_write_verb("create"),
        "write invariant: internal error — 'create' must be classified as write"
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
        for verb in READ_VERB_NAMES {
            assert!(!is_write_verb(verb), "read verb '{}' classified as write", verb);
        }
    }

    #[test]
    fn test_forbidden_verb_classification() {
        assert!(!is_forbidden_verb("create"), "create must NOT be forbidden");
        for verb in FORBIDDEN_WRITE_VERBS {
            assert!(is_forbidden_verb(verb));
        }
    }

    #[test]
    fn test_create_passes_create_only_assertion() {
        assert_create_only("create");
    }

    #[test]
    fn test_read_verbs_pass_create_only_assertion() {
        for verb in READ_VERB_NAMES {
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
    #[should_panic(expected = "zero-write invariant violated")]
    fn test_write_verb_panics_read_only() {
        assert_read_only("create");
    }

    #[test]
    fn test_invoke_br_read_builds_command() {
        let cmd = invoke_br_read(ReadVerb::List, &["--json"]);
        assert_eq!(cmd.get_program(), "br");
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["list", "--json"]);
    }

    #[test]
    fn test_invoke_br_create_builds_command() {
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
}
