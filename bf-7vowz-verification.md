[cargo-remote] uncommitted changes detected — running locally
[cargo-remote] falling back to local (CPUQuota=200%, MemoryMax=6G)
warning: type `PatternCategory` is more private than the item `DetectedPattern::category`
  --> hoop-daemon/src/reflection_detector.rs:88:5
   |
88 |     pub category: PatternCategory,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ field `DetectedPattern::category` is reachable at visibility `pub`
   |
note: but type `PatternCategory` is only usable at visibility `pub(self)`
  --> hoop-daemon/src/reflection_detector.rs:60:1
   |
60 | enum PatternCategory {
   | ^^^^^^^^^^^^^^^^^^^^
   = note: `#[warn(private_interfaces)]` on by default

warning: function `openapi_router` is never used
    --> hoop-daemon/src/lib.rs:1293:4
     |
1293 | fn openapi_router() -> Router<DaemonState> {
     |    ^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `load_hoop_config` is never used
    --> hoop-daemon/src/lib.rs:3812:10
     |
3812 | async fn load_hoop_config() -> Option<hoop_schema::HoopConfig> {
     |          ^^^^^^^^^^^^^^^^

warning: function `check_and_emit_capacity_alert` is never used
    --> hoop-daemon/src/lib.rs:4079:4
     |
4079 | fn check_and_emit_capacity_alert() -> anyhow::Result<()> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: field `session_id` is never read
   --> hoop-daemon/src/capacity.rs:358:5
    |
355 | struct ParsedPrompt {
    |        ------------ field in this struct
...
358 |     session_id: String,
    |     ^^^^^^^^^^
    |
    = note: `ParsedPrompt` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: function `get_opencode_limits` is never used
   --> hoop-daemon/src/capacity.rs:472:4
    |
472 | fn get_opencode_limits() -> OpenCodePromptLimits {
    |    ^^^^^^^^^^^^^^^^^^^

warning: field `session_subpath` is never read
   --> hoop-daemon/src/capacity.rs:526:5
    |
522 | struct GeminiAccountPaths {
    |        ------------------ field in this struct
...
526 |     session_subpath: String,
    |     ^^^^^^^^^^^^^^^
    |
    = note: `GeminiAccountPaths` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: field `rpm_limit` is never read
  --> hoop-daemon/src/capacity.rs:55:13
   |
51 |     pub struct GeminiQuotaLimits {
   |                ----------------- field in this struct
...
55 |         pub rpm_limit: Option<u64>,
   |             ^^^^^^^^^
   |
   = note: `GeminiQuotaLimits` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: struct `QuotaLimit` is never constructed
  --> hoop-daemon/src/capacity.rs:60:12
   |
60 |     struct QuotaLimit {
   |            ^^^^^^^^^^

warning: field `subpath` is never read
   --> hoop-daemon/src/sessions.rs:557:5
    |
555 | pub struct GeminiSessionPath {
    |            ----------------- field in this struct
556 |     /// Subpath to sessions (e.g., "tmp" or "sessions")
557 |     subpath: String,
    |     ^^^^^^^
    |
    = note: `GeminiSessionPath` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: constant `MAX_UNASSIGNED_SESSIONS` is never used
   --> hoop-daemon/src/sessions.rs:763:7
    |
763 | const MAX_UNASSIGNED_SESSIONS: usize = 100;
    |       ^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `STITCH_CLOSED_THRESHOLD_SECONDS` is never used
  --> hoop-daemon/src/stitch_percentile_index.rs:72:7
   |
72 | const STITCH_CLOSED_THRESHOLD_SECONDS: i64 = 300;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `hoop-daemon` (lib) generated 12 warnings
warning: variable does not need to be mutable
   --> hoop-mcp/tests/forbidden_worker_steering.rs:115:9
    |
115 |     let mut state = McpServerState::new("test-actor".to_string())
    |         ----^^^^^
    |         |
    |         help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> hoop-mcp/tests/forbidden_worker_steering.rs:148:9
    |
148 |     let mut state = McpServerState::new("test-actor".to_string())
    |         ----^^^^^
    |         |
    |         help: remove this `mut`

warning: unused variable: `bucket`
  --> hoop-cli/src/restore.rs:73:5
   |
73 |     bucket: &str,
   |     ^^^^^^ help: if this is intentional, prefix it with an underscore: `_bucket`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `object_key`
  --> hoop-cli/src/restore.rs:74:5
   |
74 |     object_key: &str,
   |     ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_object_key`

warning: unused variable: `socket_path`
  --> hoop-cli/src/script.rs:78:13
   |
78 |         let socket_path = format!("{}/.hoop/control.sock", home);
   |             ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_socket_path`

warning: unused variable: `name`
   --> hoop-cli/src/skills.rs:624:24
    |
624 | fn get_previous_sha256(name: &str) -> Result<Option<String>> {
    |                        ^^^^ help: if this is intentional, prefix it with an underscore: `_name`

warning: field `schema_version` is never read
  --> hoop-cli/src/config.rs:33:5
   |
32 | struct ConfigResponse {
   |        -------------- field in this struct
33 |     schema_version: String,
   |     ^^^^^^^^^^^^^^
   |
   = note: `ConfigResponse` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `validate_workspace` is never used
   --> hoop-cli/src/projects.rs:391:8
    |
391 | pub fn validate_workspace(path: &Path) -> Result<PathBuf> {
    |        ^^^^^^^^^^^^^^^^^^

warning: field `script` is never read
  --> hoop-cli/src/script.rs:42:5
   |
41 | struct ScriptRunResponse {
   |        ----------------- field in this struct
42 |     script: String,
   |     ^^^^^^
   |
   = note: `ScriptRunResponse` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: field `name` is never read
  --> hoop-cli/src/script.rs:63:5
   |
62 | struct ScriptManifest {
   |        -------------- field in this struct
63 |     name: String,
   |     ^^^^
   |
   = note: `ScriptManifest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: structure field `DNSName` should have a snake case name
  --> hoop-cli/src/init.rs:36:5
   |
36 |     DNSName: String,
   |     ^^^^^^^ help: convert the identifier to snake case: `dnsname`
   |
   = note: `#[warn(non_snake_case)]` (part of `#[warn(nonstandard_style)]`) on by default

warning: `hoop-mcp` (test "forbidden_worker_steering") generated 2 warnings (run `cargo fix --test "forbidden_worker_steering" -p hoop-mcp` to apply 2 suggestions)
warning: `hoop` (bin "hoop" test) generated 9 warnings (run `cargo fix --bin "hoop" -p hoop --tests` to apply 4 suggestions)
   Compiling hoop-daemon v0.1.0 (/home/coding/HOOP/hoop-daemon)
error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:657:49
    |
657 |                 events in prop::collection::vec(event_strategy, 0..20)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:363:17
    |
363 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields

error[E0063]: missing fields `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields in initializer of `DraftRow`
   --> hoop-daemon/tests/draft_queue_invariants.rs:506:17
    |
506 |     let draft = hoop_daemon::fleet::DraftRow {
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `abandoned_at`, `last_autosave_at`, `opened_at` and 2 other fields

error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:732:33
    |
732 |                 split_pos in 0..valid_event.len()
    |                                 ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:735:31
    |
735 |                 let chunk1 = &valid_event[..split_pos];
    |                               ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:736:31
    |
736 |                 let chunk2 = &valid_event[split_pos..];
    |                               ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:765:72
    |
765 |                 let split_at_boundary = split_pos == 0 || split_pos == valid_event.len();
    |                                                                        ^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0434]: can't capture dynamic environment in a fn item
   --> hoop-daemon/tests/property_invariants.rs:821:49
    |
821 |                 events in prop::collection::vec(event_strategy, 0..10)
    |                                                 ^^^^^^^^^^^^^^
    |
    = help: use the `|| { ... }` closure form instead

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:670:36
    |
670 |                     let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:678:28
    |
678 |                 let file = File::open(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:828:36
    |
828 |                     let mut file = File::create(&events_path).unwrap();
    |                                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:838:32
    |
838 |                     let file = File::open(path).unwrap();
    |                                ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:867:28
    |
867 |             let mut file = File::create(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:874:24
    |
874 |             let file = File::open(&events_path).unwrap();
    |                        ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:881:28
    |
881 |             let mut file = File::create(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:887:24
    |
887 |             let file = File::open(&events_path).unwrap();
    |                        ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:906:28
    |
906 |             let mut file = File::create(&events_path).unwrap();
    |                            ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

error[E0433]: cannot find type `File` in this scope
   --> hoop-daemon/tests/property_invariants.rs:916:20
    |
916 |         let file = File::open(&events_path).unwrap();
    |                    ^^^^ use of undeclared type `File`
    |
help: consider importing this struct
    |
588 +     use std::fs::File;
    |

warning: unused import: `HashMap`
  --> hoop-daemon/tests/property_invariants.rs:76:24
   |
76 | use std::collections::{HashMap, HashSet};
   |                        ^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused doc comment
   --> hoop-daemon/tests/property_invariants.rs:87:5
    |
 87 | /     /// Proptest: event timestamps are monotonically non-decreasing per stream
 88 | |     ///
 89 | |     /// Property: For any event stream, when read sequentially from disk,
 90 | |     /// the timestamps must never decrease within the same bead/worker stream.
...   |
101 | |     /// 2. Then try reducing the time gaps between events
102 | |     /// 3. Finally try simplifying the event types (e.g., Claim → Claim)
    | |_____-------------------------------------------------------------------^
    |       |
    |       rustdoc does not generate documentation for macro invocations
    |
    = help: to document an item produced by a macro, the macro must produce the documentation as part of its expansion
    = note: `#[warn(unused_doc_comments)]` (part of `#[warn(unused)]`) on by default

warning: unused doc comment
   --> hoop-daemon/tests/property_invariants.rs:130:5
    |
130 | /     /// Proptest: worker events maintain causal ordering
131 | |     ///
132 | |     /// Property: For any worker, its events must follow the causal chain:
133 | |     /// Claim → Dispatch → (Complete | Fail | Timeout | Crash)
...   |
143 | |     /// 1. Minimal event sequence (e.g., just ["Complete"] without Claim/Dispatch)
144 | |     /// 2. Minimal time deltas
    | |_____-------------------------^
    |       |
    |       rustdoc does not generate documentation for macro invocations
    |
    = help: to document an item produced by a macro, the macro must produce the documentation as part of its expansion

warning: unused imports: `EventTailerConfig`, `EventTailer`, and `TailerEvent`
   --> hoop-daemon/tests/property_invariants.rs:211:35
    |
211 |         use hoop_daemon::events::{EventTailer, EventTailerConfig, NeedleEvent, TailerEvent};
    |                                   ^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^               ^^^^^^^^^^^

warning: unused import: `tokio::runtime::Runtime`
   --> hoop-daemon/tests/property_invariants.rs:214:13
    |
214 |         use tokio::runtime::Runtime;
    |             ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused doc comment
   --> hoop-daemon/tests/property_invariants.rs:294:5
    |
294 | /     /// Proptest: Stitch status derivation is pure (same inputs → same output)
295 | |     ///
296 | |     /// Property: Given the same StitchContext, derive_status() always returns
297 | |     /// the same StitchStatus, regardless of how many times it's called.
...   |
308 | |     /// 2. Simple bead types (Task vs Review)
309 | |     /// 3. Minimal time differences
    | |_____------------------------------^
    |       |
    |       rustdoc does not generate documentation for macro invocations
    |
    = help: to document an item produced by a macro, the macro must produce the documentation as part of its expansion

warning: unused doc comment
   --> hoop-daemon/tests/property_invariants.rs:384:5
    |
384 | /     /// Proptest: status priority order is invariant
385 | |     ///
386 | |     /// Property: The priority order (InProgress > AwaitingReview > Quiet) is
387 | |     /// always respected, regardless of bead order or timing.
...   |
397 | |     /// 1. Minimal bead sets that violate priority (e.g., 1 claimed + 1 review)
398 | |     /// 2. Minimal timing differences
    | |_____--------------------------------^
    |       |
    |       rustdoc does not generate documentation for macro invocations
    |
    = help: to document an item produced by a macro, the macro must produce the documentation as part of its expansion

warning: unused doc comment
   --> hoop-daemon/tests/property_invariants.rs:476:5
    |
476 | /     /// Proptest: quiet days is monotonic with time
477 | |     ///
478 | |     /// Property: As time passes without activity, the "days" counter in
479 | |     /// StitchStatus::Quiet must monotonically increase.
...   |
489 | |     /// 1. Minimal day sequences (e.g., day 0 → day 1)
490 | |     /// 2. Minimal activity changes
    | |_____------------------------------^
    |       |
    |       rustdoc does not generate documentation for macro invocations
    |
    = help: to document an item produced by a macro, the macro must produce the documentation as part of its expansion

warning: unused import: `ParsedEvent`
   --> hoop-daemon/tests/property_invariants.rs:589:44
    |
589 |     use hoop_daemon::events::{NeedleEvent, ParsedEvent};
    |                                            ^^^^^^^^^^^

warning: cannot test inner items
   --> hoop-daemon/tests/property_invariants.rs:654:9
    |
654 | /         proptest! {
655 | |             #[test]
656 | |             fn proptest_replay_equals_live_inner(
657 | |                 events in prop::collection::vec(event_strategy, 0..20)
...   |
704 | |         }
    | |_________^
    |
    = note: `#[warn(unnameable_test_items)]` on by default
    = note: this warning originates in the attribute macro `test` which comes from the expansion of the macro `proptest` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: cannot test inner items
   --> hoop-daemon/tests/property_invariants.rs:729:9
    |
729 | /         proptest! {
730 | |             #[test]
731 | |             fn proptest_replay_handles_partial_lines_inner(
732 | |                 split_pos in 0..valid_event.len()
...   |
783 | |         }
    | |_________^
    |
    = note: this warning originates in the attribute macro `test` which comes from the expansion of the macro `proptest` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: cannot test inner items
   --> hoop-daemon/tests/property_invariants.rs:818:9
    |
818 | /         proptest! {
819 | |             #[test]
820 | |             fn proptest_replay_is_idempotent_inner(
821 | |                 events in prop::collection::vec(event_strategy, 0..10)
...   |
856 | |         }
    | |_________^
    |
    = note: this warning originates in the attribute macro `test` which comes from the expansion of the macro `proptest` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0063`.
error: could not compile `hoop-daemon` (test "draft_queue_invariants") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error[E0599]: no method named `lines` found for struct `std::io::BufReader<R>` in the current scope
   --> hoop-daemon/tests/property_invariants.rs:258:28
    |
258 |         for line in reader.lines() {
    |                            ^^^^^
    |
   --> /rustc/59807616e1fa2540724bfbac14d7976d7e4a3860/library/std/src/io/mod.rs:2702:7
    |
    = note: the method is available for `std::io::BufReader<std::fs::File>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `BufRead` which provides `lines` is implemented but not in scope; perhaps you want to import it
    |
 85 +     use std::io::BufRead;
    |
help: there is a method `byte_lines` with a similar name
    |
258 |         for line in reader.byte_lines() {
    |                            +++++

error[E0277]: the trait bound `StitchStatus: Hash` is not satisfied
   --> hoop-daemon/tests/property_invariants.rs:576:21
    |
576 |             results.insert(ctx.derive_status());
    |                     ^^^^^^ the trait `Hash` is not implemented for `StitchStatus`
    |
note: required by a bound in `std::collections::HashSet::<T, S, A>::insert`
   --> /rustc/59807616e1fa2540724bfbac14d7976d7e4a3860/library/std/src/collections/hash/set.rs:995:4

error[E0382]: use of moved value: `status2`
   --> hoop-daemon/tests/property_invariants.rs:380:29
    |
375 |             let status2 = ctx.derive_status();
    |                 ------- move occurs because `status2` has type `StitchStatus`, which does not implement the `Copy` trait
...
379 |             prop_assert_eq!(status1, status2, "First and second calls differ");
    |                                      ------- value moved here
380 |             prop_assert_eq!(status2, status3, "Second and third calls differ");
    |                             ^^^^^^^ value used here after move
    |
help: consider cloning the value if the performance cost is acceptable
    |
379 |             prop_assert_eq!(status1, status2.clone(), "First and second calls differ");
    |                                             ++++++++

warning: unused variable: `worker_strategy`
   --> hoop-daemon/tests/property_invariants.rs:627:13
    |
627 |         let worker_strategy = "[a-z]{3,10}";
    |             ^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_worker_strategy`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `bead_strategy`
   --> hoop-daemon/tests/property_invariants.rs:628:13
    |
628 |         let bead_strategy = "[a-z]{3,10}-[0-9]{3}";
    |             ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_bead_strategy`

warning: unused variable: `event_strategy`
   --> hoop-daemon/tests/property_invariants.rs:630:13
    |
630 |         let event_strategy = prop_oneof![
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_strategy`

warning: unused variable: `valid_event`
   --> hoop-daemon/tests/property_invariants.rs:727:13
    |
727 |         let valid_event = r#"{"event":"claim","ts":"2026-04-21T18:42:10Z","worker":"alpha","bead":"bd-1"}"#;
    |             ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_valid_event`

warning: unused variable: `event_strategy`
   --> hoop-daemon/tests/property_invariants.rs:802:13
    |
802 |         let event_strategy = prop_oneof![
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_event_strategy`

Some errors have detailed explanations: E0277, E0382, E0433, E0434, E0599.
For more information about an error, try `rustc --explain E0277`.
warning: `hoop-daemon` (test "property_invariants") generated 17 warnings
error: could not compile `hoop-daemon` (test "property_invariants") due to 19 previous errors; 17 warnings emitted

---

## SUMMARY

### Test Execution Status: BLOCKED - Compilation Failed

The `beads_deletion_http` tests **did not run** because the test suite failed to compile.

### Intended Tests (from beads_deletion_http.rs)

Three test functions were intended to run:
1. `test_beads_deletion_readyz_degraded` - Verify /readyz reports degraded when .beads is deleted
2. `test_beads_deletion_sibling_events_continue` - Verify sibling projects continue serving events during degradation  
3. `test_readyz_response_format` - Verify /readyz response format is correct

### Blocking Compilation Errors

**Total compilation failures: 21 errors across 2 test files**

#### `property_invariants.rs` - 19 errors
- **E0434 (6x)**: Proptest strategies cannot capture dynamic environment (needs `|| { ... }` closure form)
- **E0433 (9x)**: Missing `File` type import (`use std::fs::File;`)
- **E0599**: Missing `BufRead` trait for `.lines()` method
- **E0277**: `StitchStatus` doesn't implement `Hash` trait needed for `HashSet`
- **E0382**: Use of moved value `status2` (needs `.clone()`)

#### `draft_queue_invariants.rs` - 2 errors  
- **E0063 (2x)**: `DraftRow` struct missing 5 fields: `abandoned_at`, `last_autosave_at`, `opened_at`, and 2 others

### Root Cause

Stale test fixtures - production structs like `DraftRow` gained new fields that test initializers were never updated for, and missing imports in `property_invariants.rs`.

### Impact

Since `cargo test` compiles all tests in the `hoop-daemon` test target before running any, compilation errors in **unrelated test files** (`property_invariants.rs`, `draft_queue_invariants.rs`) blocked execution of the target `beads_deletion_http` tests.

### Warnings (non-blocking)

- 12 dead_code warnings in `hoop-daemon` lib
- 2 unused_mut warnings in `hoop-mcp` tests  
- 9 unused_variables/unused_imports warnings in `hoop-cli` tests

### Conclusion

No test execution results available - tests were blocked by compilation failures in other test files. The bead's acceptance criteria (capturing pass/fail status, error messages, panics) cannot be met because the tests never reached execution phase.
