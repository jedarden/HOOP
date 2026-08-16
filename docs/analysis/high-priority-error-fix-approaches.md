# High-priority compilation errors: root causes and fix approaches

**Date:** 2026-08-16
**Source inventory:** [`docs/build-logs/bf-5fkx4-complete-error-catalog.json`](../build-logs/bf-5fkx4-complete-error-catalog.json)
**Prioritization:** [`docs/build-logs/error-prioritization-analysis.md`](../build-logs/error-prioritization-analysis.md)

## Scope and selection

The source inventory is a historical snapshot of 89 blocking compiler diagnostics from
2026-07-04. Its metadata says that those diagnostics were subsequently resolved, so
this document records the engineering diagnosis and remediation sequence for the
snapshot; it is not a claim that all 89 errors are present in the current checkout.

The prioritization document groups diagnostics by error family, but its stated
`CRITICAL (19)` count does not agree with the concrete IDs it lists: those ID lists
contain 22 unique critical records (the `Clone` record for ID 64 is also discussed in
the ownership group). The selection below uses the concrete catalog IDs. It covers all
22 records in the critical architectural groups, plus IDs 1–8 from the first high
priority family. The remaining ConfigWatcher call sites (IDs 9–16) have exactly the
same root cause and are covered by the same family-level fix.

The dominant failure is an obsolete hand-built `DaemonState` test fixture in
`api_stitch_decompose.rs`. Production constructors and state fields were made more
explicit, but the fixture continued to use zero-argument constructors, undeclared
locals, and moves of values that should be shared through `Arc`. Fixing that fixture
and adding one reusable factory should remove most of the critical cluster at once.

## Selected critical records: state construction and dependency injection

| ID | Location / diagnostic | Root cause | Fix approach | Complexity | Dependencies / unlocks |
|---:|---|---|---|---|---|
| 48 | `api_stitch_decompose.rs:1214` — `ProjectSupervisor::new` called with 0 arguments | The test fixture predates the constructor's explicit dependency-injection API. The supervisor now needs event channels, shared stores, shutdown, cost/vector services, scripts path, and stuck detector. | Build the dependencies in a test-state factory, then pass the nine required handles to `ProjectSupervisor::new`. Keep each shared object in its canonical `Arc` wrapper. | **Multi-file fixture refactor** | Foundational; unblocks IDs 56, 57, 69, 70, 83, 88 and the state ownership errors. |
| 49 | `api_stitch_decompose.rs:1220` — `CostAggregator::new` called with 0 arguments | Pricing loading became path-dependent and fallible, but the fixture still assumes an in-memory/default constructor. | Create an isolated temporary pricing path (or a checked-in fixture), call the current constructor, and propagate the `Result` from the test factory. Do not hide a filesystem failure with an arbitrary production path. | **Small fixture refactor** | Depends on the shared test factory; unblocks ID 40 and supervisor construction. |
| 50 | `api_stitch_decompose.rs:1222` — `UploadRegistry::new` called with 0 arguments | Upload initialization now requires `UploadConfig` so it can validate and establish its storage allowlist. | Construct `UploadConfig` with a temporary upload directory and test limits, create the directory, and propagate constructor errors. Reuse the same temp root for cleanup. | **Small fixture refactor** | Depends on test temp-root setup; unblocks ID 41 and state construction. |
| 60 | `api_stitch_decompose.rs:1234` — `WorkerAckMonitor::new` called with 0 arguments | The monitor is filesystem-backed and its constructor is fallible/parameterized; the fixture assumes a pure value with no watcher directory. | Use the test-specific directory constructor (`with_dir`) when available, or create the required ack directory and call the current fallible constructor. Return setup errors from the test helper. | **Small fixture refactor** | Depends on temp-root setup; unblocks supervisor/state construction and prevents tests from touching the operator's `~/.hoop`. |
| 68 | `api_stitch_decompose.rs:1242` — `ProjectsRegistry::new` called with 0 arguments | Project discovery/config loading moved behind an explicit registry constructor, while the test has no project root or registry inputs in scope. | Create a registry from an isolated projects/config fixture using the current constructor. If the registry is intentionally shared, store the resulting handle in the factory rather than cloning the concrete registry. | **Multi-file fixture refactor** | Depends on IDs 69 and 78; unblocks ID 74 and the redaction-policy setup. |
| 71 | `api_stitch_decompose.rs:1244` — `StuckDetector::new` called with 0 arguments | The test's expected constructor contract diverged from the detector configuration API. The detector must receive the default or test-specific configuration map. | Select `with_config`/`with_config_map` with deterministic short test thresholds, or adapt to the current constructor. Keep the detector behind `Arc<Mutex<_>>` where the state model requires shared mutation. | **Small fixture refactor** | Depends on ID 70; unblocks supervisor construction. |
| 74 | `api_stitch_decompose.rs:1247` — `RedactionPolicyState::new` called with 0 arguments | Redaction policy resolution is derived from global config plus the project registry; a default value would skip policy semantics. | Create a valid `HoopConfig` and isolated `ProjectsRegistry`, then call `RedactionPolicyState::new(&global_config, projects_registry)`. Use a constructor rather than adding a blanket `Default` implementation for policy state. | **Small fixture refactor** | Depends on IDs 68, 75, and 78; unblocks the complete state fixture. |
| 77 | `api_stitch_decompose.rs:1249` — `ResolvedConfig::new` called with 0 arguments | Resolved configuration requires the source/config input so every value retains attribution; the test tries to bypass resolution with no input. | Resolve a minimal valid config through the production resolver (or add a deliberately named test fixture constructor). Avoid `Default` for a type whose purpose is to represent resolved configuration provenance. | **Multi-file fixture refactor** | Depends on ID 78 and the config fixture; unblocks `DaemonState` creation. |
| 84 | `api_stitch_decompose.rs:1255` — `ShutdownCoordinator::new` called with 0 arguments | Shutdown coordination was changed to receive its sender/coordinator dependency, but the fixture neither creates nor retains that channel. | Create the test shutdown channel/coordinator before constructing state and pass the required handle. Retain the sender only for tests that exercise shutdown; otherwise use a named discarded receiver to make ownership explicit. | **Small fixture refactor** | Depends on ID 86; unblocks final state assembly and avoids moving a coordinator out of the fixture. |

## Selected critical records: missing values, ownership, and sharing

| ID | Location / diagnostic | Root cause | Fix approach | Complexity | Dependencies / unlocks |
|---:|---|---|---|---|---|
| 56 | `api_stitch_decompose.rs:1215` — `bead_tx` not found | The fixture uses an event sender in a constructor argument without creating it in the local setup scope. | Create one typed broadcast channel at the start of the factory, retain its sender in `DaemonState`, and clone the sender for `WorkerRegistry`/`ProjectSupervisor` as required. Use the receiver only when a test observes events. | **Small fixture refactor** | First step for ID 48; unlocks supervisor and worker-registry construction. |
| 57 | `api_stitch_decompose.rs:1216` — `session_tx` not found | Same stale-fixture pattern as ID 56 for the session event stream. | Create a typed session broadcast channel alongside `bead_tx`; pass cloned senders to consumers and keep one sender in state. Centralizing channel creation prevents mismatched channel types. | **Small fixture refactor** | Paired with ID 56; unlocks ID 48. |
| 69 | `api_stitch_decompose.rs:1242` — `projects_dir` not found | The registry constructor now needs a project/config root, but the fixture never established one. | Derive the path from the test's `TempDir`, create the minimum project layout, and pass that path through the factory. Do not use `/home` or the real projects file in a unit test. | **Small fixture refactor** | Supplies the input for ID 68 and indirectly ID 78. |
| 70 | `api_stitch_decompose.rs:1243` — `stuck_detector` not found | A detector is passed into the supervisor/state wiring without a local instance. This is a missing initialization-order problem, not a missing global. | Construct the detector before either dependent object, wrap it once in `Arc<Mutex<_>>`, then clone the wrapper into `DaemonState` and `ProjectSupervisor`. | **Small fixture refactor** | Pairs with ID 71; unlocks ID 48. |
| 75 | `api_stitch_decompose.rs:1247` — `global_config` not found | Redaction policy setup references configuration that was never created in the fixture. | Create a named `HoopConfig` fixture (preferably the same config used to build `ResolvedConfig`) before policy construction and pass it by reference. | **Small fixture refactor** | Supplies ID 74 and keeps policy/config behavior consistent. |
| 78 | `api_stitch_decompose.rs:1249` — `projects_registry` not found | The fixture constructs or intends to construct a registry inline, then references a local that does not exist. This also obscures whether the registry is owned or shared. | Bind the registry result once, propagate its error, and pass a shared handle or an intentional clone according to the registry's API. Use that same instance for redaction policy and daemon state. | **Small fixture refactor** | Depends on ID 68/69; unlocks IDs 74 and 77. |
| 83 | `api_stitch_decompose.rs:1254` — `scripts_dir` not found | `ProjectSupervisor` gained a scripts directory for event-triggered scripts, but the test never supplied a path. | Create a temp scripts directory in the fixture, pass it to the supervisor, and leave it empty unless the test explicitly exercises scripts. | **One-file fixture change** | Supplies ID 48; isolates script discovery from the host. |
| 86 | `api_stitch_decompose.rs:1257` — `shutdown_tx` not found | Shutdown sender creation was omitted when the coordinator API changed. | Create the correctly typed channel before coordinator construction; store the sender only where the test needs to trigger shutdown and pass the coordinator handle elsewhere. | **Small fixture refactor** | Supplies ID 84 and helps eliminate IDs 61–63. |
| 88 | `api_stitch_decompose.rs:1259` — `beads` not found | The supervisor and daemon state both need the shared bead projection, but the fixture references a nonexistent local. | Create one `Arc<RwLock<Vec<Bead>>>` in the factory and clone the `Arc` into every consumer. Do not create independent vectors, which would make the test pass while projections diverge. | **Small fixture refactor** | Supplies ID 48 and is required before ownership fixes can be checked. |
| 61 | `api_stitch_decompose.rs:1234` — mutable borrow of `state.shutdown` overlaps | A partially assembled `state` value is borrowed while another field/constructor tries to consume or mutate the same field. This is a construction-order/field-borrow issue. | Stop constructing dependents from fields of a partially initialized state. Build `shutdown` as a local handle first, clone the `Arc` for each consumer, then construct `DaemonState` once. | **Multi-file fixture refactor** | Depends on IDs 56–88's factory inputs; fixing it validates the ownership model for all shared state. |
| 62 | `api_stitch_decompose.rs:1235` — use of moved `state.shutdown` | The coordinator is moved into one consumer and then reused in the state literal. The new state model expects shared ownership, not repeated ownership of the concrete value. | Keep `Arc<ShutdownCoordinator>` in a local, pass `shutdown.clone()` to the supervisor, and move only the final `Arc` into state. | **One-line after factory refactor** | Depends on ID 84/86; unlocks construction of the supervisor and state together. |
| 63 | `api_stitch_decompose.rs:1236` — use of moved `state.worker_registry` | The worker registry is similarly moved into a dependent and then needed by `DaemonState`. | Construct the `Arc<WorkerRegistry>` once and clone the `Arc` for every consumer. Do not derive `Clone` on the registry merely to make the fixture compile. | **One-line after factory refactor** | Depends on event channels (IDs 56–57); unlocks the state initializer. |
| 64 | `api_stitch_decompose.rs:1238` — `CostAggregator: Clone` bound not satisfied | The fixture attempts to clone the concrete service while the ownership design shares it through `Arc<RwLock<_>>`. Adding `Clone` to a stateful service would duplicate pricing/bucket state and hide the design error. | Create one `Arc<RwLock<CostAggregator>>` and clone that wrapper into state and supervisor. Propagate constructor errors before wrapping it. | **Small ownership refactor** | Depends on ID 49; unlocks ID 48 and removes pressure to add unsafe/semantic `Clone` impls. |

## Selected high-priority records: ConfigWatcher call sites

IDs 1–8 are separate compiler records but one API-evolution defect: the
`reload_config` signature gained `agent_config_changed_tx`, while older test calls
still pass four arguments. The same remediation applies to IDs 9–16.

| ID | Location | Root cause and fix approach | Complexity | Dependencies / unlocks |
|---:|---|---|---|---|
| 1 | `config_watcher.rs:591` | Four-argument call omits `agent_config_changed_tx`. Create the sender holder in the test setup and pass `agent_config_changed_tx.clone()` as the fifth argument. | **One-line call-site fix** plus shared setup | Requires the sender fixture; unlocks this test path. |
| 2 | `config_watcher.rs:617` | Same stale call signature. Add the fifth argument from the common sender holder; preserve the receiver if the test asserts an agent-config event. | **One-line call-site fix** | Same shared setup as ID 1. |
| 3 | `config_watcher.rs:642` | Same stale call signature. Pass the cloned sender holder rather than constructing a new channel per call, so event assertions observe the actual reload. | **One-line call-site fix** | Depends on common sender setup; unlocks the next reload scenario. |
| 4 | `config_watcher.rs:679` | Same stale call signature. Add the sender argument and keep the async call's ownership model unchanged. | **One-line call-site fix** | Same shared setup as ID 1. |
| 5 | `config_watcher.rs:715` | Same stale call signature. Add `agent_config_changed_tx.clone()`; use `None` inside the holder only when the scenario intentionally disables notification. | **One-line call-site fix** | Depends on explicit test intent for notification vs. no notification. |
| 6 | `config_watcher.rs:751` | Same stale call signature. Pass the common holder so reload and agent notification remain coupled as production code expects. | **One-line call-site fix** | Same shared setup as ID 1. |
| 7 | `config_watcher.rs:787` | Same stale call signature. Add the fifth parameter and retain the existing config/event fixtures. | **One-line call-site fix** | Same shared setup as ID 1. |
| 8 | `config_watcher.rs:832` | Same stale call signature. Add the fifth parameter; after all calls are migrated, compile-check IDs 9–16 with the same mechanical change. | **One-line call-site fix** | Completing the family unlocks the full ConfigWatcher test module and validates agent-config hot reload. |

## Recommended dependency-aware sequence

1. Add a single `test_daemon_state()`/fixture builder with a `TempDir`, typed
   broadcast channels, shared bead store, temp scripts/uploads/pricing paths,
   shutdown channel, detector, registry, config, and resolved config. This addresses
   the missing-value records (IDs 56, 57, 69, 70, 75, 78, 83, 86, 88).
2. Update the builder to call the current fallible/parameterized constructors (IDs
   48–50, 60, 68, 71, 74, 77, 84), propagating setup errors instead of using
   production paths or blanket `unwrap` calls.
3. Pass `Arc` handles, rather than concrete services, to all consumers and construct
   `DaemonState` only after its dependencies exist (IDs 61–64). This is the central
   architectural fix; adding `Clone` derives to stateful concrete types is not.
4. Migrate all 16 ConfigWatcher call sites (IDs 1–16) to the five-argument API and
   add a test for the notification channel's `Some(sender)` and `None` behavior.
5. Run the targeted `api_stitch_decompose`, `config_watcher`, and daemon test targets
   before the full workspace test. A successful compile is necessary but not
   sufficient: the fixture must also prove that shared state and config-change
   notifications are observed by the intended consumers.

## Complexity and dependency summary

| Work package | Selected records | Complexity | What it unlocks |
|---|---|---|---|
| Shared daemon-state fixture and temp resources | 56, 57, 69, 70, 75, 78, 83, 86, 88 | **Medium / multi-file test refactor** | All constructor and ownership diagnostics in `api_stitch_decompose.rs` |
| Constructor/API migration | 48, 49, 50, 60, 68, 71, 74, 77, 84 | **Medium** | A compilable, semantically valid `DaemonState` fixture |
| Ownership and shared-handle correction | 61, 62, 63, 64 | **Medium** | Supervisor/state initialization without moves or misleading `Clone` impls |
| ConfigWatcher signature migration | 1–16 | **Low / repetitive** | Config hot-reload test coverage and agent-config notification checks |
