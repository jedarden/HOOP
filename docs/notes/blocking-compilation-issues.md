# HOOP blocking issues: final compilation and phase-gate synthesis

**Assembled:** 2026-08-16
**Primary historical catalog:** [`bf-5fkx4-complete-error-catalog.json`](../build-logs/bf-5fkx4-complete-error-catalog.json)
**Scope:** Rust compiler diagnostics and the verification blockers that determine the Phase 1 exit gate.

## Summary

The canonical compiler snapshot contains **89 unique blocking diagnostics** from
2026-07-04. Its own metadata says those diagnostics were fixed after the snapshot,
so the 89 records are a historical catalog and remediation map, not evidence that
all 89 errors remain in the checkout today. A separate build-status artifact reports
`cargo build` with zero errors and 102 warnings. The current repository guide still
reports that the Phase 1 gate is open because the full test target and the strict
Clippy run have separate failures; those are verification blockers, not additional
records to add to the 89-error historical total.

| Measure | Count / status | Interpretation |
|---|---:|---|
| Historical unique compiler diagnostics | **89** | Complete record set in the `bf-5fkx4` catalog. |
| Largest file cluster | **44** | `hoop-daemon/src/api_stitch_decompose.rs`; the stale hand-built `DaemonState` fixture. |
| `ConfigWatcher::reload_config` call sites | **16** | One missing `agent_config_changed_tx` argument at each site. |
| `CapacityMeterConfig` initializers | **11** | Test fixtures omit the same newly required fields. |
| Build-status artifact | **0 errors / 102 warnings** | `bf-2sih1` reports no compile blocker in that snapshot. |
| Phase 1 verification gate | **Open** | The guide reports test compilation and `clippy -- -D warnings` still need resolution; `hoop status --json` passes. |

## Current checkout verification (2026-08-16)

The current working tree is not a clean historical baseline: it contains
uncommitted changes from other work. Against that checkout,
`cargo test --workspace --no-run` exited **101** and found **one current
compilation blocker**. The command reached test-target compilation far enough
to build `hoop-daemon`, then stopped on this error; the two `hoop-mcp` warnings
shown in the same run are non-blocking.

### P0-CURRENT-1 — `notify::Watcher` trait is not in scope

| Field | Detail |
|---|---|
| File and line | [`hoop-daemon/src/api_skills.rs:1148`](../../hoop-daemon/src/api_skills.rs:1148) (the `.watch(...)` call); the import is at line 36 |
| Command | `cargo test --workspace --no-run` |
| Exit status | `101` |
| Error code | `E0599` |
| Root cause | The current uncommitted edit changed `use notify::{RecursiveMode, Watcher};` at line 36 to `use notify::RecursiveMode;`. `RecommendedWatcher` implements the `notify::Watcher` trait, but Rust does not make trait methods available until the trait is imported or the call is fully qualified. |
| Suggested fix | Restore the trait import, preferably as `use notify::{RecursiveMode, Watcher};`, then rerun `cargo test --workspace --no-run`. An equivalent fix is a fully-qualified `notify::Watcher::watch(&mut watcher, ...)` call, but the import is consistent with the existing method-call style. |

Full compiler diagnostic captured from the verification run:

```text
error[E0599]: no method named `watch` found for struct `INotifyWatcher` in the current scope
    --> hoop-daemon/src/api_skills.rs:1148:14
     |
1147 | /         watcher
1148 | |             .watch(&skills_dir, RecursiveMode::NonRecursive)
     | |_____________-^^^^^
     |
     ::: /home/coding/.cargo/registry/src/index.crates.io-1949da8c6fb5b557f/notify-6.1.1/src/lib.rs:340:8
     |
 340 |       fn watch(&mut self, path: &Path, recursive_mode: RecursiveMode) -> Result<()>;
     |          ---- the method is available for `INotifyWatcher` here
     |
     = help: items from traits can only be used if the trait is in scope
help: there is a method `unwatch` with a similar name, but with different arguments
    --> /home/coding/.cargo/registry/src/index.crates.io-1949da8c6fb5b557f/notify-6.1.1/src/lib.rs:348:5
     |
 348 |     fn unwatch(&mut self, path: &Path) -> Result<()>;
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
help: trait `Watcher` which provides `watch` is implemented but not in scope; perhaps you want to import it
     |
  36 + use notify::Watcher;
     |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `hoop-daemon` (lib) due to 1 previous error
```

This P0 entry is a **current** blocker and is intentionally kept separate
from the 89-record historical catalog below. If another worker changes
`api_skills.rs`, rerun the command before relying on this snapshot.

### Highest-priority items

1. **Repair the `api_stitch_decompose.rs` test-state factory.** Forty-four records
   cluster around obsolete constructor calls, missing channels and paths, fallible
   constructors, missing fields, and attempts to clone concrete stateful services.
   One dependency-aware fixture factory addresses most of this cluster at once.
2. **Migrate all 16 ConfigWatcher reload calls.** Add the shared agent-config
   notification sender and test both notification-enabled and notification-disabled
   paths.
3. **Update the 11 capacity fixtures.** Supply explicit, isolated values for the
   account file, GCP quota configuration, Gemini directories, and OpenCode
   directories.
4. **Finish the integration-harness and isolated API migrations.** These include
   `Utf8Bytes` WebSocket text values, `workspace`, `attachments_count`, `stash_sha`,
   and the renamed `DaemonHandle::temp_dir` field.
5. **Only after compilation is green, close the Phase 1 verification loop.** Run
   the complete workspace tests, strict Clippy, and the non-interactive status
   check on the same commit as required by [plan §10](../plan/plan.md#10-milestones).

### Count and taxonomy note

The earlier prioritization note reports severity tiers of 19 critical, 38 high,
28 medium, and 4 low, but those tiers do not reconcile cleanly with the concrete
record IDs; its critical discussion names 22 distinct records. This final note uses
the `blocking_errors[]` records as the source of truth, counts each Rust error code
once per record, and uses root-cause concentration for ordering. The compiler-code
counts below therefore supersede the older severity totals.

## Categorized error catalog

The following table is an exact grouping of all 89 catalog records by their
`error_type` field. ID ranges in the detailed tables below are expanded by listed
locations or symbols; no diagnostic record is intentionally omitted.

| Rust code | Count | Main family | Primary remediation |
|---|---:|---|---|
| `E0061` | 25 | Wrong argument count / constructor drift | Update call sites to the current API; share the new dependency handles. |
| `E0063` | 17 | Missing struct or enum fields | Update fixtures with explicit values or a shared fixture constructor. |
| `E0277` | 16 | Missing `Clone` bounds in state assembly | Share state with `Arc`/locks; do not add misleading `Clone` implementations to stateful services. |
| `E0308` | 11 | Type and `Result` mismatches | Use the API's current types and propagate or assert fallible setup intentionally. |
| `E0425` | 9 | Missing local values | Create dependencies in deterministic fixture setup and bind them once. |
| `E0369` | 3 | Missing comparison trait | Derive or implement `PartialEq` for `RiskSeverity`. |
| `E0599` | 3 | Removed `Default`/associated API | Use the current constructor or add a semantically valid default only where appropriate. |
| `E0382` | 2 | Use-after-move | Clone shared handles, not concrete stateful values. |
| `E0433` | 1 | Missing dependency | Add the required development dependency (`rand`). |
| `E0502` | 1 | Conflicting borrow | Construct locals before assembling `DaemonState`; avoid borrowing partially initialized state. |
| `E0609` | 1 | Renamed/removed field access | Use `DaemonHandle::temp_dir`. |
| **Total** | **89** |  |  |

### E0061 — argument count and constructor signature mismatches (25)

| IDs | Location(s) | Diagnostic / affected API | Fix direction |
|---|---|---|---|
| 1–16 | `config_watcher.rs`: 591, 617, 642, 679, 715, 751, 787, 832, 873, 915, 956, 997, 1038, 1079, 1122, 1165 | `ConfigWatcher::reload_config` takes five arguments; each call supplies four and omits `agent_config_changed_tx`. | Create one typed sender holder in the fixture and pass a clone at all 16 sites. |
| 48 | `api_stitch_decompose.rs:1214` | `ProjectSupervisor::new` takes 9 arguments; fixture supplies 0. | Build the supervisor dependencies in a test factory and pass the nine current handles. |
| 49 | `api_stitch_decompose.rs:1220` | `CostAggregator::new` takes 1 argument; fixture supplies 0. | Use an isolated temporary pricing/config path and propagate the constructor result. |
| 50 | `api_stitch_decompose.rs:1222` | `UploadRegistry::new` takes 1 argument; fixture supplies 0. | Supply a temporary `UploadConfig` and propagate setup errors. |
| 60 | `api_stitch_decompose.rs:1234` | `WorkerAckMonitor::new` takes 1 argument; fixture supplies 0. | Create an isolated ack directory and use the current fallible/parameterized constructor. |
| 68 | `api_stitch_decompose.rs:1242` | `ProjectsRegistry::new` takes 2 arguments; fixture supplies 0. | Build the registry from a temporary project/config fixture and bind the result once. |
| 71 | `api_stitch_decompose.rs:1244` | `StuckDetector::new` takes 1 argument; fixture supplies 0. | Supply deterministic test configuration and share the detector handle. |
| 74 | `api_stitch_decompose.rs:1247` | `RedactionPolicyState::new` takes 2 arguments; fixture supplies 0. | Pass a valid `HoopConfig` and the shared projects registry. |
| 77 | `api_stitch_decompose.rs:1249` | `ResolvedConfig::new` takes 1 argument; fixture supplies 0. | Resolve a minimal valid config through the production resolver. |
| 84 | `api_stitch_decompose.rs:1255` | `ShutdownCoordinator::new` takes 1 argument; fixture supplies 0. | Create the test shutdown channel/coordinator before final state assembly. |

The representative diagnostic is:

```text
error[E0061]: this function takes 5 arguments but 4 arguments were supplied
  --> hoop-daemon/src/config_watcher.rs:591:13
   |
   | an argument of type Arc<Mutex<Option<broadcast::Sender<AgentConfigChanged>>>> is missing
```

### E0063 — missing fields in initializers (17)

| IDs | Location(s) | Initializer | Missing data | Fix direction |
|---|---|---|---|---|
| 17 | `tests/integration_harness.rs:269` | `Bead` | `workspace` | Set the fixture workspace explicitly. |
| 18 | `api_preview.rs:621` | `PreviewRequest` | `attachments_count` | Set zero for no attachments or compute the real count. |
| 19 | `api_stitch_decompose.rs:1203` | `DaemonState` | `br_semaphore`, `br_semaphore_target_permits` | Initialize the bounded bead subprocess semaphore in the factory. |
| 20–30 | `capacity.rs`: 2457, 2503, 2573, 2774, 2851, 2913, 3058, 3111, 3203, 3227, 3267 | `CapacityMeterConfig` | `accounts_file`, `gcp_quota_config`, `gemini_dirs`, `opencode_dirs` | Centralize test defaults; use empty directory lists and `None` where the scenario does not exercise the provider. |
| 31 | `dictated_notes.rs:776` | `DictatedNote` | `draft_id`, `synthesis_result` | Initialize both optional workflow fields to `None` in the basic fixture. |
| 32 | `redaction_policy.rs:546` | `HoopConfig` | `embedding`, `redaction` | Include the current optional config sections in the test config. |
| 33 | `load_test.rs:182` | `NeedleEvent::Fail` | `stash_sha` | Supply a deterministic synthetic SHA matching the current variant type. |

### E0277 — missing `Clone` bounds during state assembly (16)

| IDs | Location(s) | Type reported | Correct ownership approach |
|---|---|---|---|
| 58 | `api_stitch_decompose.rs:1217` | `WorkerRegistry` | Construct once and share the intended `Arc` handle. |
| 59, 67 | `api_stitch_decompose.rs:1218, 1241` | `VectorIndex` | Share the index handle required by the state model; do not duplicate index state. |
| 64 | `api_stitch_decompose.rs:1238` | `CostAggregator` | Use one `Arc<RwLock<CostAggregator>>`; a concrete `Clone` would duplicate pricing state. |
| 65 | `api_stitch_decompose.rs:1239` | `UploadRegistry` | Share the registry handle created from the isolated upload config. |
| 66 | `api_stitch_decompose.rs:1240` | `WorkerAckMonitor` | Share the monitor handle; preserve its filesystem-backed state. |
| 72 | `api_stitch_decompose.rs:1245` | `StuckDetector` | Construct once, then clone its `Arc<Mutex<_>>` wrapper. |
| 73 | `api_stitch_decompose.rs:1246` | `ProjectsRegistry` | Bind one registry result and share the API-approved wrapper. |
| 76 | `api_stitch_decompose.rs:1248` | `RedactionPolicyState` | Construct from config and registry; share the state wrapper rather than deriving `Clone`. |
| 79 | `api_stitch_decompose.rs:1249` | `ResolvedConfig` | Keep the resolved configuration in its intended shared container. |
| 80 | `api_stitch_decompose.rs:1250` | `ProjectSupervisor` | Do not clone a supervisor task tree; retain the intended handle/reference. |
| 81–82 | `api_stitch_decompose.rs:1251–1253` | `broadcaster::Sender<BeadEvent>` and `Sender<SessionEvent>` | Create typed channels once and clone the sender handles. |
| 85 | `api_stitch_decompose.rs:1256` | `ShutdownCoordinator` | Share the coordinator handle; do not move the concrete coordinator twice. |
| 87 | `api_stitch_decompose.rs:1258` | `Arc<RwLock<Vec<Bead>>>` | Re-check this diagnostic after the fixture refactor; the wrapper should already be shareable. |
| 89 | `api_stitch_decompose.rs:1260` | `DaemonState` | Build state once after all dependencies exist; callers should receive a reference/handle, not clone the whole state. |

### E0308 — type and `Result` mismatches (11)

| IDs | Location(s) | Mismatch | Fix direction |
|---|---|---|---|
| 34 | `api_stitch_decompose.rs:1205` | Expected `tokio::time::Instant`, found `std::time::Instant`. | Use the expected Tokio type or an intentional conversion. |
| 35–39 | `tests/integration_harness.rs`: 862, 1105, 1202, 1214, 1222 | Expected `Utf8Bytes`, found `String` for WebSocket `Message::Text`. | Convert the text with the current tungstenite API, e.g. `text.into()`. |
| 40 | `api_stitch_decompose.rs:1220` | Expected `CostAggregator`, found `Result<CostAggregator, Error>`. | Handle the fallible constructor before wrapping the value. |
| 41 | `api_stitch_decompose.rs:1222` | Expected `UploadRegistry`, found `Result<UploadRegistry, Error>`. | Propagate or explicitly assert the isolated test setup error. |
| 42 | `api_stitch_decompose.rs:1232` | Expected `WorkerAckMonitor`, found `Result<WorkerAckMonitor, Error>`. | Handle the current fallible constructor. |
| 43, 44 | `heartbeats.rs`: 935, 1089 | Expected `()`, found `Result<(), _>` in property-test assertions. | Assert the result or return it from the property test according to the harness contract. |

### E0425 — missing values in scope (9)

| IDs | Location | Missing value | Fix direction |
|---|---|---|---|
| 56 | `api_stitch_decompose.rs:1215` | `bead_tx` | Create a typed bead-event broadcast channel in the fixture. |
| 57 | `api_stitch_decompose.rs:1216` | `session_tx` | Create the paired session-event channel and retain its sender. |
| 69 | `api_stitch_decompose.rs:1242` | `projects_dir` | Derive it from the test `TempDir`; never use a real operator path. |
| 70 | `api_stitch_decompose.rs:1243` | `stuck_detector` | Construct the configured detector before dependent state. |
| 75 | `api_stitch_decompose.rs:1247` | `global_config` | Create one named minimal `HoopConfig` fixture. |
| 78 | `api_stitch_decompose.rs:1249` | `projects_registry` | Bind the registry result once and reuse the shared instance. |
| 83 | `api_stitch_decompose.rs:1254` | `scripts_dir` | Create an empty temporary scripts directory. |
| 86 | `api_stitch_decompose.rs:1257` | `shutdown_tx` | Create the correctly typed shutdown channel before the coordinator. |
| 88 | `api_stitch_decompose.rs:1259` | `beads` | Create one shared `Arc<RwLock<Vec<Bead>>>` for all consumers. |

### Remaining compiler-code groups

| Rust code | IDs | Location / diagnostic | Fix direction |
|---|---|---|---|
| `E0369` | 45–47 | `risk_patterns.rs:662, 666, 670`: `RiskSeverity` cannot be compared with `==`. | Derive or implement `PartialEq`; add a focused regression test. |
| `E0599` | 51–52 | `api_stitch_decompose.rs:1230, 1237`: `ResolvedConfig::default()` and `RedactionPolicyState::default()` are unavailable. | Use explicit production constructors; add `Default` only if an all-zero/default state is semantically valid. |
| `E0599` | 53 | `redaction.rs:498`: `SecretPattern::default_secret_patterns()` is unavailable. | Migrate to the current default-pattern API or construct the configured pattern list explicitly. |
| `E0382` | 62–63 | `api_stitch_decompose.rs:1235–1236`: moved `state.shutdown` and `state.worker_registry`. | Move only final state ownership; clone `Arc` handles before passing them to dependents. |
| `E0433` | 55 | `integration_harness.rs:192`: crate/module `rand` not found. | Add `rand` to the appropriate dependency section and keep the use test-only if applicable. |
| `E0502` | 61 | `api_stitch_decompose.rs:1234`: mutable borrow of `state.shutdown` overlaps another borrow. | Stop constructing dependents from a partially initialized state; build locals first. |
| `E0609` | 54 | `tests/integration_harness.rs:602`: `DaemonHandle` has no `_temp_dir` field. | Use the renamed `temp_dir` field. |

### Record coverage check

This is the complete ID partition used to audit the catalog:

```text
E0061: 1-16, 48-50, 60, 68, 71, 74, 77, 84
E0063: 17-33
E0277: 58, 59, 64-67, 72, 73, 76, 79-82, 85, 87, 89
E0308: 34-44
E0369: 45-47
E0382: 62-63
E0425: 56, 57, 69, 70, 75, 78, 83, 86, 88
E0433: 55
E0502: 61
E0599: 51-53
E0609: 54
```

## Prioritized analysis

### P0 — obsolete `DaemonState` fixture (44 records; highest leverage)

The dominant cluster is the hand-built fixture in
`hoop-daemon/src/api_stitch_decompose.rs:1203–1260`. Production types evolved to
require explicit dependencies, fallible resource setup, shared state, and new
configuration fields, while the fixture retained zero-argument constructors and
inline moves from a partially initialized `state` value.

The root cause is therefore **fixture architecture drift**, not 44 independent
production defects. A correct fix should introduce one dependency-aware test
factory, for example:

```rust
fn test_daemon_state() -> anyhow::Result<(DaemonState, TestResources)> {
    let temp = tempfile::tempdir()?;
    let (bead_tx, _bead_rx) = tokio::sync::broadcast::channel(32);
    let (session_tx, _session_rx) = tokio::sync::broadcast::channel(32);

    // Create isolated pricing, uploads, ack, scripts, and projects paths.
    // Construct fallible services before assembling state.
    // Share stateful services through their intended Arc/RwLock wrappers.
    // Initialize semaphore, shutdown, registry, and config fields explicitly.

    Ok((state, TestResources { temp }))
}
```

The factory must not use the operator's home directory, add blanket `Clone`
implementations merely to satisfy the old fixture, or create independent copies
of projections that production expects to share. The detailed dependency map is in
[`high-priority-error-fix-approaches.md`](../analysis/high-priority-error-fix-approaches.md).

### P1 — ConfigWatcher API migration (16 records; broad but mechanical)

`reload_config` gained `agent_config_changed_tx`, but every old test call still
uses the four-argument form. This is a single API migration repeated at 16 sites.
Use one sender holder so tests observe the same notification channel as production.
After migration, test both `Some(sender)` and `None` behavior; otherwise a
compiling fixture could still silently lose agent-config reload events.

### P1 — capacity configuration fixture migration (11 records)

`CapacityMeterConfig` gained provider-specific discovery and quota fields. The
production change is intentional, but test initializers should express whether a
scenario uses those providers. A shared test constructor with explicit empty lists,
`None` quota configuration, and an isolated accounts path reduces repetition and
prevents future field drift.

### P1 — integration transport and schema fixtures (7 records)

The integration harness is affected by the schema's required `workspace`, the
tungstenite `Utf8Bytes` change, and the `temp_dir` rename. These are isolated from
the daemon-state architecture and can be fixed after the shared fixture is
compiling. The WebSocket conversion should be applied consistently at message
construction boundaries, not by weakening the message type.

### P2 — isolated API and test-harness repairs (remaining records)

The remaining records are small in code volume but should be handled deliberately:

- initialize `PreviewRequest`, `DictatedNote`, `HoopConfig`, and `NeedleEvent::Fail`
  with their current fields;
- update the heartbeat property tests for the current `Result` contract;
- add `PartialEq` for `RiskSeverity`;
- migrate the redaction default-pattern call;
- add the missing `rand` dependency;
- correct the renamed integration-harness field.

These fixes should follow the API's current semantics rather than restoring old
defaults that hide missing configuration.

## Recommended fix sequence

| Step | Work package | Why this order maximizes impact | Exit check |
|---:|---|---|---|
| 0 | Establish the baseline from a clean, identified commit. Separate current worktree edits from the historical `bf-5fkx4` snapshot. | Prevents mixing old diagnostics with concurrent changes or stale generated artifacts. | Record `cargo check`, `cargo test`, and Clippy commands plus commit SHA. |
| 1 | Build `test_daemon_state()` with `TempDir`, channels, shared bead storage, config/resolver, projects registry, pricing/uploads/ack/scripts paths, shutdown, and semaphore fields. | Addresses the central 44-record dependency cluster and supplies inputs for the ownership fixes. | The fixture itself compiles with no concrete-service `Clone` workarounds. |
| 2 | Complete constructor/result/ownership migration in `api_stitch_decompose.rs`. | Constructor errors, missing locals, `E0308`, `E0382`, `E0502`, and most `E0277` records are coupled. | Targeted daemon test target compiles and exercises shared state. |
| 3 | Migrate all 16 `ConfigWatcher::reload_config` calls. | One systematic API change restores the hot-reload test module. | Targeted ConfigWatcher tests compile and verify notification delivery. |
| 4 | Update the 11 `CapacityMeterConfig` fixtures through a shared test helper. | Removes repeated schema drift and keeps provider-specific defaults explicit. | Capacity tests compile; provider-disabled cases remain deterministic. |
| 5 | Repair integration transport/schema fixtures and isolated API records. | These are mostly independent and become easier to diagnose once core test setup builds. | Targeted integration, preview, dictated-note, heartbeat, redaction, and risk-pattern tests compile. |
| 6 | Run `cargo test --workspace` and classify any remaining failures as compile-time or runtime. | Compilation success is necessary but does not prove behavior. | All unit and integration tests pass, as required by the phase gate. |
| 7 | Run `cargo clippy --workspace -- -D warnings` and address the separate lint backlog. | Clippy is an independent Phase 1 gate; do not hide it in the compiler-error count. | Zero warnings treated as errors. |
| 8 | Run `hoop status --json | jq .` and the Phase 1 success-criteria tests. | Completes the plan's non-interactive and functional gate on the same commit. | Valid JSON, correct exit behavior, and automated `testrepo/` coverage. |

## Phase 1 gate and related findings

The [canonical plan §10](../plan/plan.md#10-milestones) requires all of the
following on the same commit before Phase 2 work begins:

```text
cargo test --workspace
cargo clippy --workspace -- -D warnings
hoop status --json | jq .
Phase 1 success-criteria tests against testrepo/
```

The historical compiler catalog and the Phase 1 gate are related but not
interchangeable:

| Evidence | What it says | How to use it |
|---|---|---|
| [`bf-5fkx4-complete-error-catalog.json`](../build-logs/bf-5fkx4-complete-error-catalog.json) | 89 compile diagnostics existed in a 2026-07-04 snapshot and were later resolved. | Use for the complete error catalog and root-cause sequence. |
| [`bf-2sih1-issues.json`](../build-logs/bf-2sih1-issues.json) | Its build snapshot has 0 errors and 102 warnings; it marks no blocking issues in that run. | Confirms that warnings-only build output is not itself the 89-error blocker. |
| [`bf-xibss-clippy-warnings.md`](bf-xibss-clippy-warnings.md) | An older Clippy investigation recorded 54 compile errors and 74 warnings around OpenAPI `ToSchema` and unused code. | Historical evidence only; do not add these counts to 89. |
| Repository guide / Phase 1 gate | The current guide reports test compilation still failing and strict Clippy reporting 90 errors across 39 files, while `hoop status --json` passes. | Treat test compilation and Clippy as separate current gate classes until freshly verified on a clean commit. |
| [`error_message_catalog_analysis.md`](../test-coverage/error_message_catalog_analysis.md) | 1,192 strings extracted statically from tests. | Test-message inventory, not compiler failures. |

The working tree was not a clean verification baseline while this documentation
was assembled, so the status statements above retain their source dates and
provenance. A fresh gate run must be recorded before declaring Phase 1 complete.

## Related non-compiler issue: `claimed_at` timestamp formats

The `claimed_at` analyses identify a **medium-severity data-format consistency
issue**, not a Rust compilation blocker. HOOP and the bead CLI can encounter valid
RFC3339 values with different precision, legacy SQLite timestamps, empty values, or
malformed values. The recommended sequence is to add format-variant tests and
validate at write time, then consider canonical normalization. Keep this separate
from the 89 compiler records so a runtime/data-quality concern is not mistaken for
a build failure. See [`claimed_at` root-cause analysis](../analysis/claimed_at_format_mismatch_root_cause.md)
and the [timestamp parsing investigation](claimed_at-parsing-investigation-bf-5wxp7.md).

## Source and technical-note index

- [Canonical implementation plan](../plan/plan.md), especially [§10 milestones](../plan/plan.md#10-milestones) and [§14 testing strategy](../plan/plan.md#14-testing-strategy).
- [Historical complete 89-error catalog](../build-logs/bf-5fkx4-complete-error-catalog.json).
- [Historical blocking-error report](../compilation-errors-blocking.md).
- [Prioritization analysis](../build-logs/error-prioritization-analysis.md).
- [High-priority root causes and fix approaches](../analysis/high-priority-error-fix-approaches.md).
- [Test-failure analysis showing compilation-gated tests](../verification/test-failure-analysis.md).
- [Clippy/OpenAPI investigation](bf-xibss-clippy-warnings.md).
- [Static test error-message inventory](../test-coverage/error_message_catalog_analysis.md).
