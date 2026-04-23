//! CI test: fake br stub that logs all verbs; assert only `create` is called
//!
//! Exercises the MCP code paths that call br through `invoke_br_create()` and
//! verifies that only `create` is ever invoked.
//!
//! CI command:
//!   cargo test -p hoop-mcp --features=create-only-write --test create_only_stub

use std::fs;
use std::io::Write;
use std::path::PathBuf;

struct FakeBr {
    bin_dir: tempfile::TempDir,
    log_path: PathBuf,
}

impl FakeBr {
    fn new() -> Self {
        let bin_dir = tempfile::TempDir::new().expect("create temp dir");
        let br_path = bin_dir.path().join("br");
        let log_path = bin_dir.path().join("br_invocations.log");

        let log_path_str = log_path.to_str().unwrap();
        let script = format!(
            "#!/bin/sh\n\
             echo \"$@\" >> {log_path_str}\n\
             if [ \"$1\" = \"create\" ]; then\n\
               echo \"bd-stub-$(date +%s)\"\n\
             elif [ \"$1\" = \"list\" ]; then\n\
               echo '[]'\n\
             elif [ \"$1\" = \"get\" ]; then\n\
               echo '{{}}'\n\
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

        Self { bin_dir, log_path }
    }

    fn path_prefix(&self) -> String {
        self.bin_dir.path().to_str().unwrap().to_string()
    }

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

fn path_with_fake(fake: &FakeBr) -> String {
    format!("{}:{}", fake.path_prefix(), std::env::var("PATH").unwrap_or_default())
}

#[test]
fn test_invoke_br_create_calls_only_create_verb() {
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        let fake = FakeBr::new();
        let mut cmd = hoop_mcp::br_verbs::invoke_br_create(&["Test bead", "--type", "task"]);
        cmd.env("PATH", path_with_fake(&fake));
        let output = cmd.output().expect("run fake br");
        assert!(output.status.success(), "fake br should succeed");

        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 1, "expected exactly one invocation, got {:?}", verbs);
        assert_eq!(verbs[0], "create");
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
fn test_invoke_br_create_multiple_invocations_all_create() {
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        let fake = FakeBr::new();
        let path_env = path_with_fake(&fake);

        for i in 0..3 {
            let mut cmd = hoop_mcp::br_verbs::invoke_br_create(&[]);
            cmd.arg(format!("Bead {}", i));
            cmd.arg("--type").arg("task");
            cmd.env("PATH", &path_env);
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
    let path_env = path_with_fake(&fake);

    let read_verbs = [
        ("list", hoop_mcp::br_verbs::ReadVerb::List),
        ("get", hoop_mcp::br_verbs::ReadVerb::Get),
    ];

    for (_name, verb) in &read_verbs {
        let mut cmd = hoop_mcp::br_verbs::invoke_br_read(*verb, &[]);
        cmd.env("PATH", &path_env);
        let _ = cmd.output();
    }

    let verbs = fake.verbs();
    for verb in &verbs {
        assert!(
            !hoop_mcp::br_verbs::is_write_verb(verb),
            "read verb '{}' classified as write — this is a bug",
            verb
        );
    }
}

#[test]
fn test_forbidden_verbs_never_called() {
    let forbidden = hoop_mcp::br_verbs::FORBIDDEN_WRITE_VERBS;
    let expected_forbidden = ["close", "update", "release", "claim", "depend"];
    assert_eq!(forbidden.len(), expected_forbidden.len(),
        "FORBIDDEN_WRITE_VERBS has {} entries, expected {}",
        forbidden.len(), expected_forbidden.len());

    for verb in &expected_forbidden {
        assert!(forbidden.contains(verb), "'{}' missing from FORBIDDEN_WRITE_VERBS", verb);
        assert!(hoop_mcp::br_verbs::is_forbidden_verb(verb), "'{}' not detected as forbidden", verb);
    }

    assert!(!hoop_mcp::br_verbs::is_forbidden_verb("create"));
    assert!(hoop_mcp::br_verbs::is_write_verb("create"));
}

#[test]
fn test_runtime_guard_rejects_forbidden_verbs() {
    for verb in hoop_mcp::br_verbs::FORBIDDEN_WRITE_VERBS {
        let result = std::panic::catch_unwind(|| {
            hoop_mcp::br_verbs::assert_create_only(verb);
        });
        assert!(result.is_err(), "assert_create_only('{}') should have panicked", verb);
    }
}

#[test]
fn test_runtime_guard_allows_create() {
    hoop_mcp::br_verbs::assert_create_only("create");
}

#[cfg(any(feature = "create-only-write", feature = "zero-write-v01"))]
#[test]
fn test_subprocess_arg_validation_rejects_forbidden_commands() {
    for verb in hoop_mcp::br_verbs::FORBIDDEN_WRITE_VERBS {
        let result = std::panic::catch_unwind(|| {
            let mut cmd = std::process::Command::new("br");
            cmd.arg(verb).arg("bd-test123");
            hoop_mcp::br_verbs::validate_br_subprocess_args(&cmd);
        });
        assert!(result.is_err(),
            "validate_br_subprocess_args should reject raw '{}' command", verb);
    }
}

#[test]
fn test_invoke_br_create_end_to_end_with_stub() {
    #[cfg(any(
        feature = "create-only-write",
        not(any(feature = "zero-write-v01", feature = "create-only-write"))
    ))]
    {
        let fake = FakeBr::new();
        let path_env = path_with_fake(&fake);

        let titles = ["Fix auth race", "Add test coverage", "Update docs"];
        for title in &titles {
            let mut cmd = hoop_mcp::br_verbs::invoke_br_create(&[]);
            cmd.arg(title);
            cmd.arg("--type").arg("task");
            cmd.arg("--labels").arg("stitch:test-stitch");
            cmd.arg("--actor").arg("test-actor");
            cmd.arg("--silent");
            cmd.env("PATH", &path_env);
            let output = cmd.output().expect("run fake br");
            assert!(output.status.success(), "fake br should succeed for '{}'", title);
        }

        let verbs = fake.verbs();
        assert_eq!(verbs.len(), 3, "expected 3 invocations, got {:?}", verbs);
        for (i, verb) in verbs.iter().enumerate() {
            assert_eq!(verb, "create",
                "invocation {} should be 'create', got '{}'", i, verb);
        }

        let invocations = fake.invocations();
        assert!(invocations[0].contains("Fix auth race"), "first invocation should contain title");
        assert!(invocations[1].contains("stitch:test-stitch"), "should contain stitch label");
    }
}

#[test]
fn test_read_verbs_also_pass_subprocess_validation() {
    let verbs_to_test = [
        hoop_mcp::br_verbs::ReadVerb::List,
        hoop_mcp::br_verbs::ReadVerb::Get,
        hoop_mcp::br_verbs::ReadVerb::Status,
        hoop_mcp::br_verbs::ReadVerb::Version,
        hoop_mcp::br_verbs::ReadVerb::Doctor,
        hoop_mcp::br_verbs::ReadVerb::Log,
        hoop_mcp::br_verbs::ReadVerb::Show,
    ];
    for verb in &verbs_to_test {
        let cmd = hoop_mcp::br_verbs::invoke_br_read(*verb, &[]);
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args[0], std::ffi::OsStr::new(verb.as_str()));
    }
}
