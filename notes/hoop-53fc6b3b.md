# Targeted load-test integration compile

- HEAD: `da15b53bf452dc779fa451e268fc7a465e778a31`
- Extraction: `/home/coding/scratch/hoop-53fc6b3b-head.dRXexo` (removed after successful verification)
- Wrapper: `nix-shell --run` on NixOS
- Command: `cargo test -p hoop-daemon --features testing --test load_test_integration --no-run`
- Exit code: `0`
- Result: the `load_test_integration` test target compiled successfully. Cargo emitted warnings only.

Captured output tail:

```text
254 |     cost_aggregator: Arc<std::sync::RwLock<CostAggregator>>,
    |     ^^^^^^^^^^^^^^^
    |
    = note: `ProjectSupervisor` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

warning: `hoop-daemon` (lib) generated 28 warnings (run `cargo fix --lib -p hoop-daemon` to apply 4 suggestions)
warning: unused import: `PerformanceReport`
  --> hoop-daemon/tests/load_test_integration.rs:21:56
   |
21 |     measure_memory, populate_testrepo, LoadTestConfig, PerformanceReport,
   |                                                        ^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `beads`
   --> hoop-daemon/tests/integration_harness.rs:754:9
    |
754 |     let beads: Vec<hoop_daemon::Bead> = resp.json().await.expect("Failed to parse beads response");
    |         ^^^^^ help: if this is intentional, prefix it with an underscore: `_beads`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `bead_id`
    --> hoop-daemon/tests/integration_harness.rs:1317:9
     |
1317 |     let bead_id = bead["id"].as_str().expect("Bead should have an ID");
     |         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_bead_id`

warning: unused variable: `i`
    --> hoop-daemon/tests/integration_harness.rs:1354:9
     |
1354 |     for i in 0..5 {
     |         ^ help: if this is intentional, prefix it with an underscore: `_i`

warning: variable does not need to be mutable
   --> hoop-daemon/tests/load_test_integration.rs:303:22
    |
303 |                 let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    |                      ----^^^^^^^^^
    |                      |
    |                      help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: field `temp_dir` is never read
   --> hoop-daemon/tests/integration_harness.rs:585:9
    |
583 | pub struct DaemonHandle {
    |            ------------ field in this struct
584 |     pub shutdown_notify: Arc<tokio::sync::Notify>,
585 |     pub temp_dir: TempDir,
    |         ^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `hoop-daemon` (test "load_test_integration") generated 6 warnings (run `cargo fix --test "load_test_integration" -p hoop-daemon` to apply 5 suggestions)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7m 49s
  Executable tests/load_test_integration.rs (/build/target/debug/deps/load_test_integration-7a989f4ddf608c3a)

EXIT_CODE=0
```
