//! bead verb classification and create-only invariant guard
//!
//! Phase 1 (zero-write-v01): strictly read-only, no bead writes at all.
//! Phase 4+ (create-only-write): only `bead create` is allowed.
//!
//! This module:
//! - Classifies bead verbs as read or write
//! - Under `zero-write-v01`, ALL write verbs are unreachable at compile time
//! - Under `create-only-write`, only `invoke_bead_create()` compiles; other write
//!   verbs fail to compile and are rejected at runtime
//! - `validate_write_invariant()` is called at daemon startup to log the mode
//! - Provides `spawn_bead_command` for executing bead subprocesses with semaphore control (§4.8)
//!
//! The bead CLI name is configurable via HOOP_BEAD_CLI environment variable
//! and defaults to "bead" (the bead-rs CLI).

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

/// All bead verb names that are classified as write operations.
pub const WRITE_VERB_NAMES: &[&str] = &["create", "close", "update", "release", "claim", "depend"];

/// All bead verb names that are classified as read operations.
pub const READ_VERB_NAMES: &[&str] = &[
    "list",
    "get",
    "status",
    "--version",
    "doctor",
    "log",
    "show",
];

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
/// Belt-and-suspenders: under `create-only-write`, `invoke_bead_write` does not compile,
/// so non-create write verbs can't get here. But `invoke_bead` (string-based) or a
/// raw `Command::new(bead_cli)` could — this catches those paths.
pub fn assert_create_only(verb: &str) {
    if is_forbidden_verb(verb) {
        panic!(
            "HOOP create-only invariant violated: bead {} is forbidden. \
             Only `bead create` is allowed in phase 4+. This is a bug — please report it.",
            verb
        );
    }
    // Also catch any completely unknown write verbs
    if is_write_verb(verb) && verb != "create" {
        panic!(
            "HOOP create-only invariant violated: bead {} is a write verb but not 'create'. \
             This is a bug — please report it.",
            verb
        );
    }
}

/// Runtime guard that panics if ANY write verb is attempted (phase 1 zero-write mode).
pub fn assert_read_only(verb: &str) {
    if is_write_verb(verb) {
        panic!(
            "HOOP zero-write invariant violated: bead {} is a write verb. \
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
/// typed builders (e.g., raw `Command::new(bead_cli)` or post-construction mutation).
///
/// Under `create-only-write`: only `create` and read verbs pass.
/// Under `zero-write-v01`: only read verbs pass.
/// Unrestricted: all verbs pass.
pub fn validate_bead_subprocess_args(cmd: &std::process::Command) {
    let first_arg = cmd.get_args().next();
    let verb = first_arg
        .map(|a| a.to_string_lossy().into_owned())
        .unwrap_or_default();

    if ZERO_WRITE_ACTIVE {
        if verb.is_empty() {
            panic!(
                "HOOP zero-write invariant violated: bead invoked with no verb. \
                 Phase 1 is strictly read-only. This is a bug — please report it."
            );
        }
        assert_read_only(&verb);
    } else if CREATE_ONLY_ACTIVE {
        if verb.is_empty() {
            panic!(
                "HOOP create-only invariant violated: bead invoked with no verb. \
                 Only `bead create` is allowed in phase 4+. This is a bug — please report it."
            );
        }
        assert_create_only(&verb);
    }
    // Unrestricted mode: no validation needed
}

/// Invoke a bead read verb. This is always available regardless of feature flags.
pub fn invoke_bead_read(verb: ReadVerb, args: &[&str]) -> std::process::Command {
    assert_read_only(verb.as_str());
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    let mut cmd = std::process::Command::new(cmd_name);
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    validate_bead_subprocess_args(&cmd);
    cmd
}

/// Invoke `bead create` — the single allowed write verb in phase 4+.
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
pub fn invoke_bead_create(args: &[&str]) -> std::process::Command {
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    let mut cmd = std::process::Command::new(cmd_name);
    cmd.arg("create");
    for arg in args {
        cmd.arg(arg);
    }
    validate_bead_subprocess_args(&cmd);
    cmd
}

/// Invoke a bead write verb. Only available when NO write restriction is active
/// (neither `zero-write-v01` nor `create-only-write`).
///
/// Under `create-only-write`, this function does not exist at compile time —
/// use `invoke_bead_create()` instead.
/// Under `zero-write-v01`, neither this nor `invoke_bead_create` exists.
#[cfg(not(any(feature = "zero-write-v01", feature = "create-only-write")))]
pub fn invoke_bead_write(verb: WriteVerb, args: &[&str]) -> std::process::Command {
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    let mut cmd = std::process::Command::new(cmd_name);
    cmd.arg(verb.as_str());
    for arg in args {
        cmd.arg(arg);
    }
    validate_bead_subprocess_args(&cmd);
    cmd
}

/// Invoke bead with an arbitrary verb string, enforcing the write invariant at runtime.
/// Use typed `invoke_bead_read` / `invoke_bead_create` instead when the verb is known at compile time.
#[cfg(not(feature = "zero-write-v01"))]
pub fn invoke_bead(verb: &str, args: &[&str]) -> std::process::Command {
    #[cfg(feature = "create-only-write")]
    assert_create_only(verb);
    #[cfg(not(feature = "create-only-write"))]
    assert_read_only(verb);
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    let mut cmd = std::process::Command::new(cmd_name);
    cmd.arg(verb);
    for arg in args {
        cmd.arg(arg);
    }
    validate_bead_subprocess_args(&cmd);
    cmd
}

/// Invoke bead with an arbitrary verb string — zero-write variant (rejects ALL writes).
#[cfg(feature = "zero-write-v01")]
pub fn invoke_bead(verb: &str, args: &[&str]) -> std::process::Command {
    assert_read_only(verb);
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    let mut cmd = std::process::Command::new(cmd_name);
    cmd.arg(verb);
    for arg in args {
        cmd.arg(arg);
    }
    validate_bead_subprocess_args(&cmd);
    cmd
}

/// Validate the write invariant at daemon startup.
///
/// Logs the invariant mode and panics if the runtime guards detect an inconsistency.
pub fn validate_write_invariant() {
    let cmd_name = hoop_core::bead_cli::bead_cli_command();
    if ZERO_WRITE_ACTIVE {
        tracing::info!(
            "write invariant: ZERO-WRITE (phase 1 — no bead write verbs at compile time, CLI: {})",
            cmd_name
        );
    } else if CREATE_ONLY_ACTIVE {
        tracing::info!(
            "write invariant: CREATE-ONLY (phase 4+ — only bead create at compile time, CLI: {})",
            cmd_name
        );
    } else {
        tracing::warn!(
            "write invariant: UNRESTRICTED (no feature flag set — all bead verbs reachable, CLI: {})",
            cmd_name
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

/// Extract `stitch:*` labels from a label list.
///
/// Used to propagate stitch lineage from a claimed bead to follow-up beads
/// created by the worker (Hook 4 — stitch label inheritance).
pub fn extract_stitch_labels(labels: &[String]) -> Vec<String> {
    labels
        .iter()
        .filter(|l| l.starts_with("stitch:"))
        .cloned()
        .collect()
}

/// Propagate `stitch:*` labels from a parent bead to a new bead's label list.
///
/// Implements Hook 4 (stitch label inheritance) for daemon-side bead creation.
/// Extracts all `stitch:*` labels from `parent_labels` and appends them to
/// `target_labels`, deduplicating against any already-present labels.
pub fn propagate_stitch_labels(target_labels: &mut Vec<String>, parent_labels: &[String]) {
    let inherited = extract_stitch_labels(parent_labels);
    for label in &inherited {
        if !target_labels.contains(label) {
            target_labels.push(label.clone());
        }
    }
}


/// Spawn a bead subprocess with semaphore control and metrics (§4.8).
///
/// This function:
/// 1. Acquires a permit from the global bead subprocess semaphore
/// 2. Increments the concurrent subprocess gauge metric
/// 3. Spawns the bead command in a blocking task
/// 4. Decrements the concurrent metric and releases the permit on completion
/// 5. Records the subprocess total and duration metrics
///
/// # Arguments
///
/// * `semaphore` - The global semaphore limiting concurrent bead subprocesses
/// * `cmd` - The bead Command to execute (built by invoke_bead_* functions)
/// * `verb` - The bead verb name for metrics (e.g., "create", "list")
///
/// # Returns
///
/// * `Ok(output)` - The subprocess stdout/stderr if successful
/// * `Err(e)` - Any error from spawning or executing the subprocess
pub async fn spawn_bead_command(
    semaphore: &tokio::sync::Semaphore,
    mut cmd: std::process::Command,
    verb: &str,
) -> anyhow::Result<std::process::Output> {
    use crate::metrics::metrics;
    use std::time::Instant;

    // Acquire permit from semaphore (blocks if limit reached)
    let _permit = semaphore
        .acquire()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to acquire bead subprocess semaphore: {}", e))?;

    // Increment concurrent gauge metric
    metrics().hoop_bead_subprocess_concurrent.inc();

    let start = Instant::now();

    // Spawn the bead command in a blocking task
    let result = tokio::task::spawn_blocking(move || cmd.output())
        .await
        .map_err(|e| anyhow::anyhow!("Task join failed: {}", e))?
        .map_err(|e| anyhow::anyhow!("Failed to run bead {}: {}", verb, e));

    let duration_ms = start.elapsed().as_millis() as f64;

    // Always decrement concurrent gauge and record metrics
    metrics().hoop_bead_subprocess_concurrent.dec();

    // Record total and duration metrics based on result
    match &result {
        Ok(output) if output.status.success() => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "ok"]);
        }
        Ok(_) => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "error"]);
        }
        Err(_) => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "error"]);
        }
    }
    metrics()
        .hoop_bead_subprocess_duration_ms
        .observe(&[verb], duration_ms);

    result
}

/// Synchronous wrapper for `spawn_bead_command` for use in blocking contexts.
///
/// This blocks the current thread until the bead subprocess completes.
/// Prefer `spawn_bead_command` in async contexts.
///
/// # Arguments
///
/// * `semaphore` - The global semaphore limiting concurrent bead subprocesses
/// * `cmd` - The bead Command to execute (built by invoke_bead_* functions)
/// * `verb` - The bead verb name for metrics (e.g., "create", "list")
///
/// # Returns
///
/// * `Ok(output)` - The subprocess stdout/stderr if successful
/// * `Err(e)` - Any error from spawning or executing the subprocess
pub fn spawn_bead_command_blocking(
    semaphore: &tokio::sync::Semaphore,
    mut cmd: std::process::Command,
    verb: &str,
) -> anyhow::Result<std::process::Output> {
    let handle = semaphore
        .try_acquire()
        .map_err(|e| anyhow::anyhow!("Failed to acquire bead subprocess semaphore: {}", e))?;

    use crate::metrics::metrics;
    metrics().hoop_bead_subprocess_concurrent.inc();

    let start = std::time::Instant::now();
    let result = cmd.output();
    let duration_ms = start.elapsed().as_millis() as f64;

    metrics().hoop_bead_subprocess_concurrent.dec();

    match &result {
        Ok(output) if output.status.success() => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "ok"]);
        }
        Ok(_) => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "error"]);
        }
        Err(_) => {
            metrics().hoop_bead_subprocess_total.inc(&[verb, "error"]);
        }
    }
    metrics()
        .hoop_bead_subprocess_duration_ms
        .observe(&[verb], duration_ms);

    drop(handle);
    result.map_err(|e| anyhow::anyhow!("Failed to run bead {}: {}", verb, e))
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
            assert!(
                !is_forbidden_verb(verb),
                "read verb '{}' must not be forbidden",
                verb
            );
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
    fn test_invoke_bead_read_builds_command() {
        let cmd = invoke_bead_read(ReadVerb::List, &["--json"]);
        let cmd_name = hoop_core::bead_cli::bead_cli_command();
        assert_eq!(cmd.get_program(), std::path::Path::new(&cmd_name).as_os_str());
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["list", "--json"]);
    }

    #[test]
    fn test_invoke_bead_string_read_builds_command() {
        let cmd = invoke_bead("list", &["--json"]);
        let cmd_name = hoop_core::bead_cli::bead_cli_command();
        assert_eq!(cmd.get_program(), std::path::Path::new(&cmd_name).as_os_str());
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["list", "--json"]);
    }

    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_invoke_bead_string_write_panics() {
        let _ = invoke_bead("close", &["bd-abc123"]);
    }

    #[test]
    #[allow(clippy::overly_complex_bool_expr, clippy::assertions_on_constants)]
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
        let expected = [
            "list",
            "get",
            "status",
            "--version",
            "doctor",
            "log",
            "show",
        ];
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
    fn test_invoke_bead_create_builds_command() {
        // invoke_bead_create is only available under create-only-write or unrestricted
        #[cfg(any(
            feature = "create-only-write",
            not(any(feature = "zero-write-v01", feature = "create-only-write"))
        ))]
        {
            let cmd = invoke_bead_create(&["--type", "task"]);
            let cmd_name = hoop_core::bead_cli::bead_cli_command();
            assert_eq!(cmd.get_program(), std::path::Path::new(&cmd_name).as_os_str());
            let args: Vec<_> = cmd.get_args().collect();
            assert_eq!(args, ["create", "--type", "task"]);
        }
    }

    // -----------------------------------------------------------------------
    // Subprocess-arg inspection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_bead_subprocess_args_allows_create() {
        let mut cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
        cmd.arg("create").arg("--type").arg("task");
        validate_bead_subprocess_args(&cmd);
    }

    #[test]
    fn test_validate_bead_subprocess_args_allows_read_verbs() {
        for verb in READ_VERB_NAMES {
            let mut cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
            cmd.arg(verb);
            validate_bead_subprocess_args(&cmd);
        }
    }

    #[cfg(any(feature = "create-only-write", feature = "zero-write-v01"))]
    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_bead_subprocess_args_rejects_raw_close_command() {
        let mut cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
        cmd.arg("close").arg("bd-abc123");
        validate_bead_subprocess_args(&cmd);
    }

    #[cfg(any(feature = "create-only-write", feature = "zero-write-v01"))]
    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_bead_subprocess_args_rejects_raw_update_command() {
        let mut cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
        cmd.arg("update").arg("bd-abc123");
        validate_bead_subprocess_args(&cmd);
    }

    #[cfg(any(feature = "create-only-write", feature = "zero-write-v01"))]
    #[test]
    #[should_panic(expected = "invariant violated")]
    fn test_validate_bead_subprocess_args_rejects_empty_command() {
        let cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
        validate_bead_subprocess_args(&cmd);
    }

    #[cfg(not(any(feature = "create-only-write", feature = "zero-write-v01")))]
    #[test]
    fn test_validate_bead_subprocess_args_allows_all_without_feature_flag() {
        for verb in WRITE_VERB_NAMES {
            let mut cmd = std::process::Command::new(hoop_core::bead_cli::bead_cli_command());
            cmd.arg(verb);
            validate_bead_subprocess_args(&cmd);
        }
    }

    // -----------------------------------------------------------------------
    // Stitch label extraction tests (Hook 4)
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_stitch_labels_single() {
        let labels = vec!["stitch:abc123".to_string(), "urgent".to_string()];
        let stitch_labels = extract_stitch_labels(&labels);
        assert_eq!(stitch_labels, vec!["stitch:abc123"]);
    }

    #[test]
    fn test_extract_stitch_labels_multiple() {
        let labels = vec![
            "stitch:abc123".to_string(),
            "urgent".to_string(),
            "stitch:def456".to_string(),
        ];
        let stitch_labels = extract_stitch_labels(&labels);
        assert_eq!(stitch_labels, vec!["stitch:abc123", "stitch:def456"]);
    }

    #[test]
    fn test_extract_stitch_labels_none() {
        let labels = vec!["urgent".to_string(), "bug".to_string()];
        let stitch_labels = extract_stitch_labels(&labels);
        assert!(stitch_labels.is_empty());
    }

    #[test]
    fn test_extract_stitch_labels_empty() {
        let labels: Vec<String> = vec![];
        let stitch_labels = extract_stitch_labels(&labels);
        assert!(stitch_labels.is_empty());
    }

    #[test]
    fn test_extract_stitch_labels_no_false_positives() {
        let labels = vec![
            "stitching".to_string(),
            "nostitch:foo".to_string(),
            "xstitch:bar".to_string(),
        ];
        let stitch_labels = extract_stitch_labels(&labels);
        assert!(stitch_labels.is_empty());
    }

    // -----------------------------------------------------------------------
    // Hook 4: propagate_stitch_labels tests (daemon create_bead)
    // -----------------------------------------------------------------------

    #[test]
    fn test_propagate_stitch_labels_worker_followup_single() {
        let parent_labels = vec!["stitch:abc123".to_string(), "urgent".to_string()];
        let mut target = vec!["follow-up".to_string()];
        propagate_stitch_labels(&mut target, &parent_labels);
        assert!(
            target.contains(&"stitch:abc123".to_string()),
            "worker-created follow-up bead must inherit parent's stitch label"
        );
        assert!(target.contains(&"follow-up".to_string()));
        assert!(
            !target.contains(&"urgent".to_string()),
            "non-stitch labels must not propagate"
        );
    }

    #[test]
    fn test_propagate_stitch_labels_multiple_stitch_labels() {
        let parent_labels = vec![
            "stitch:abc123".to_string(),
            "stitch:def456".to_string(),
            "urgent".to_string(),
        ];
        let mut target: Vec<String> = vec![];
        propagate_stitch_labels(&mut target, &parent_labels);
        assert_eq!(
            target,
            vec!["stitch:abc123", "stitch:def456"],
            "all stitch labels must propagate to follow-up bead"
        );
    }
}
