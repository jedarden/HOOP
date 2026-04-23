//! CI test: fake br stub that logs all verbs; assert only `create` is called
//!
//! This test creates a temporary directory with a fake `br` shell script that
//! logs every invocation to a file. It then exercises the HOOP code paths that
//! call br through `invoke_br_create()` and verifies that only `create` is
//! ever invoked.
//!
//! CI command:
//!   cargo test -p hoop-daemon --features=create-only-write --test create_only_stub

use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Path to the log file written by the fake br stub
struct FakeBr {
    /// Directory containing the fake `br` script
    bin_dir: tempfile::TempDir,
    /// Path to the invocation log
    log_path: PathBuf,
}

impl FakeBr {
    fn new() -> Self {
        let bin_dir = tempfile::TempDir::new().expect("create temp dir");
        let br_path = bin_dir.path().join("br");
        let log_path = bin_dir.path().join("br_invocations.log");

        // Write the fake br script: log verb + args, then output a fake bead ID
        let log_path_str = log_path.to_str().unwrap();
        let script = format!(
            "#!/bin/sh\n\
             echo \"$@\" >> {log_path_str}\n\
             # If verb is 'create', output a fake bead ID\n\
             if [ \"$1\" = \"create\" ]; then\n\
               echo \"bd-stub-$(date +%s)\"\n\
             fi\n\
             exit 0\n"
        );
        let mut f = fs::File::create(&br_path).expect("create br script");
        f.write_all(script.as_bytes()).expect("write br script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&br_path, fs::Permissions::from_mode(0o755))
                .expect("chmod br script");
        }

        Self {
            bin_dir,
            log_path,
        }
    }

    /// Get the PATH prefix that includes the fake br
    fn path_prefix(&self) -> String {
        self.bin_dir.path().to_str().unwrap().to_string()
    }

    /// Read and parse all logged invocations
    fn invocations(&self) -> Vec<String> {
        if !self.log_path.exists() {
            return vec![];
        }
        let contents = fs::read_to_string(&self.log_path).unwrap_or_default();
        contents
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    /// Extract just the verb (first arg) from each invocation
    fn verbs(&self) -> Vec<String> {
        self.invocations()
            .iter()
            .map(|line| {
                line.split_whitespace()
                    .next()
                    .unwrap_or("(empty)")
                    .to_string()
            })
            .collect()
    }
}

#[test]
fn test_invoke_br_create_calls_only_create_verb() {
    let fake = FakeBr::new();

    // invoke_br_create is only available under create-only-write or unrestricted
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        // Build a command through the create-only API
        let mut cmd = hoop_daemon::br_verbs::invoke_br_create(&["Test bead", "--type", "task"]);
        cmd.env("PATH", format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default()));

        let output = cmd.output().expect("run fake br");
        assert!(output.status.success(), "fake br should succeed");

        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 1, "expected exactly one invocation, got {:?}", verbs);
        assert_eq!(verbs[0], "create", "only 'create' verb should be called, got '{}'", verbs[0]);
    }

    #[cfg(not(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    )))]
    {
        // Under zero-write-v01, invoke_br_create doesn't exist — nothing to test here.
        // The compile-time check ensures it can't be called.
        println!("invoke_br_create not available under zero-write-v01 — test is a no-op");
    }
}

#[test]
fn test_invoke_br_create_multiple_invocations_all_create() {
    let fake = FakeBr::new();

    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        // Simulate creating multiple beads (as stitch submit would)
        for i in 0..3 {
            let mut cmd = hoop_daemon::br_verbs::invoke_br_create(&[]);
            cmd.arg(format!("Bead {}", i));
            cmd.arg("--type").arg("task");
            cmd.env("PATH", format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default()));
            let _ = cmd.output();
        }

        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
        for verb in &verbs {
            assert_eq!(verb, "create", "only 'create' verb should be called, got '{}'", verb);
        }
    }

    #[cfg(not(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    )))]
    {
        println!("invoke_br_create not available under zero-write-v01 — test is a no-op");
    }
}

#[test]
fn test_invoke_br_read_verbs_never_write() {
    let fake = FakeBr::new();

    // Test that read verbs go through the read path and never trigger write classification
    let read_verbs = [
        ("list", hoop_daemon::br_verbs::ReadVerb::List),
    ];

    let empty = "".to_string();
    for (name, verb) in &read_verbs {
        let mut cmd = hoop_daemon::br_verbs::invoke_br_read(*verb, &["--json"]);
        cmd.env("PATH", format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default()));
        let _ = cmd.output();

        // Verify the logged verb matches the read verb
        let invocations = fake.invocations();
        let last = invocations.last().unwrap_or(&empty);
        assert!(
            last.starts_with(name),
            "expected invocation to start with '{}', got '{}'",
            name,
            last
        );
    }

    let verbs = fake.verbs();
    for verb in &verbs {
        assert!(
            !hoop_daemon::br_verbs::is_write_verb(verb),
            "read verb '{}' classified as write — this is a bug",
            verb
        );
    }
}

#[test]
fn test_forbidden_verbs_never_called() {
    // This test verifies that the FORBIDDEN_WRITE_VERBS constant correctly
    // identifies all non-create write verbs, ensuring no verb slips through.
    let forbidden = hoop_daemon::br_verbs::FORBIDDEN_WRITE_VERBS;

    // These are ALL write verbs except create
    let expected_forbidden = ["close", "update", "release", "claim", "depend"];
    assert_eq!(forbidden.len(), expected_forbidden.len(),
        "FORBIDDEN_WRITE_VERBS has {} entries, expected {}",
        forbidden.len(), expected_forbidden.len());

    for verb in &expected_forbidden {
        assert!(forbidden.contains(verb), "'{}' missing from FORBIDDEN_WRITE_VERBS", verb);
        assert!(hoop_daemon::br_verbs::is_forbidden_verb(verb), "'{}' not detected as forbidden", verb);
    }

    // Verify create is NOT forbidden
    assert!(!hoop_daemon::br_verbs::is_forbidden_verb("create"));
    assert!(hoop_daemon::br_verbs::is_write_verb("create"));
}

#[test]
fn test_runtime_guard_rejects_forbidden_verbs() {
    // Verify each forbidden verb triggers the runtime guard
    for verb in hoop_daemon::br_verbs::FORBIDDEN_WRITE_VERBS {
        let result = std::panic::catch_unwind(|| {
            hoop_daemon::br_verbs::assert_create_only(verb);
        });
        assert!(result.is_err(), "assert_create_only('{}') should have panicked", verb);
        let err = result.unwrap_err();
        let msg = err.downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| err.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        assert!(msg.contains("create-only invariant violated"),
            "panic message should mention create-only invariant, got: {}", msg);
    }
}

#[test]
fn test_runtime_guard_allows_create() {
    // create should NOT panic
    hoop_daemon::br_verbs::assert_create_only("create");
}

// ---------------------------------------------------------------------------
// Subprocess-arg inspection tests (runtime belt-and-suspenders)
// ---------------------------------------------------------------------------

#[test]
fn test_subprocess_arg_validation_allows_create_command() {
    // Verify that a command built by invoke_br_create passes subprocess-arg validation
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        let fake = FakeBr::new();
        let mut cmd = hoop_daemon::br_verbs::invoke_br_create(&["Test bead", "--type", "task"]);
        cmd.env("PATH", format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default()));
        let output = cmd.output().expect("run fake br");
        assert!(output.status.success());

        // Verify the fake br logged exactly "create" as the verb
        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs[0], "create");

        // Verify the command args start with "create"
        let cmd2 = hoop_daemon::br_verbs::invoke_br_create(&["Another bead"]);
        let args: Vec<_> = cmd2.get_args().collect();
        assert_eq!(args[0], std::ffi::OsStr::new("create"),
            "invoke_br_create must produce 'create' as first arg");
    }
}

#[test]
fn test_subprocess_arg_validation_rejects_forbidden_commands() {
    // Verify that raw Command objects with forbidden verbs are rejected
    // by validate_br_subprocess_args
    for verb in hoop_daemon::br_verbs::FORBIDDEN_WRITE_VERBS {
        let result = std::panic::catch_unwind(|| {
            let mut cmd = std::process::Command::new("br");
            cmd.arg(verb).arg("bd-test123");
            hoop_daemon::br_verbs::validate_br_subprocess_args(&cmd);
        });
        assert!(result.is_err(),
            "validate_br_subprocess_args should reject raw '{}' command", verb);
    }
}

#[test]
fn test_invoke_br_create_end_to_end_with_stub() {
    // End-to-end test: exercise the full create path and verify only "create" is logged
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        let fake = FakeBr::new();
        let path_env = format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default());

        // Simulate a full stitch submit creating multiple beads
        let titles = ["Fix auth race", "Add test coverage", "Update docs"];
        for title in &titles {
            let mut cmd = hoop_daemon::br_verbs::invoke_br_create(&[]);
            cmd.arg(title);
            cmd.arg("--type").arg("task");
            cmd.arg("--labels").arg("stitch:test-stitch");
            cmd.arg("--actor").arg("test-actor");
            cmd.arg("--silent");
            cmd.env("PATH", &path_env);
            let output = cmd.output().expect("run fake br");
            assert!(output.status.success(), "fake br should succeed for '{}'", title);
        }

        // Verify ALL logged verbs are "create"
        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
        for (i, verb) in verbs.iter().enumerate() {
            assert_eq!(verb, "create",
                "invocation {} should be 'create', got '{}'", i, verb);
        }

        // Verify full invocation log includes expected args
        let invocations = fake.invocations();
        assert!(invocations[0].contains("Fix auth race"), "first invocation should contain title");
        assert!(invocations[1].contains("stitch:test-stitch"), "should contain stitch label");
    }
}

#[test]
fn test_read_verbs_also_pass_subprocess_validation() {
    // Verify read-verb commands pass subprocess-arg validation
    let verbs_to_test = [
        hoop_daemon::br_verbs::ReadVerb::List,
        hoop_daemon::br_verbs::ReadVerb::Get,
        hoop_daemon::br_verbs::ReadVerb::Status,
        hoop_daemon::br_verbs::ReadVerb::Version,
        hoop_daemon::br_verbs::ReadVerb::Doctor,
        hoop_daemon::br_verbs::ReadVerb::Log,
        hoop_daemon::br_verbs::ReadVerb::Show,
    ];
    for verb in &verbs_to_test {
        let cmd = hoop_daemon::br_verbs::invoke_br_read(*verb, &[]);
        // validate_br_subprocess_args is called internally by invoke_br_read
        // If we reach this point, validation passed
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args[0], std::ffi::OsStr::new(verb.as_str()));
    }
}
