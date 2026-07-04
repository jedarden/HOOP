# Test Failure Checklist — bf-1c0jw

**Generated:** 2026-07-04  
**Test Run Logs:** 
- `hoop-test-run-20260704-154650.log`
- `hoop-test-run-20260704-155501.log`

## Summary

**Total Compilation Errors:** 78 errors across 12 files

**By Crate:**
- `hoop-daemon`: 75 errors (lib + tests)
- `hoop-mcp`: 0 errors (warnings only)
- `hoop-cli`: 0 errors (warnings only)

## Compilation Failures

### hoop-daemon (lib)

#### api_stitch_decompose.rs (23 errors)
- [ ] `api_stitch_decompose::Arc::new(identity_cache)` - Missing `Arc` type import (E0433:L1197)
- [ ] `api_stitch_decompose::Arc::new(role_resolver)` - Missing `Arc` type import (E0433:L1198)
- [ ] `api_stitch_decompose::Arc::new(worker_registry)` - Missing `Arc` type import (E0433:L1205)
- [ ] `api_stitch_decompose::Arc::new(beads)` - Missing `Arc` type import (E0433:L1209)
- [ ] `api_stitch_decompose::Arc::new(shutdown)` - Missing `Arc` type import (E0433:L1212)
- [ ] `api_stitch_decompose::Arc::new(supervisor)` - Missing `Arc` type import (E0433:L1213)
- [ ] `api_stitch_decompose::Arc::new(projects)` - Missing `Arc` type import (E0433:L1214)
- [ ] `api_stitch_decompose::Arc::new(project_metadata)` - Missing `Arc` type import (E0433:L1215)
- [ ] `api_stitch_decompose::Arc::new(cost_aggregator)` - Missing `Arc` type import (E0433:L1219)
- [ ] `api_stitch_decompose::Arc::new(upload_registry)` - Missing `Arc` type import (E0433:L1221)
- [ ] `api_stitch_decompose::Arc::new(active_project)` - Missing `Arc` type import (E0433:L1222)
- [ ] `api_stitch_decompose::Arc::new(vector_index)` - Missing `Arc` type import (E0433:L1223)
- [ ] `api_stitch_decompose::Arc::new(resolved_config)` - Missing `Arc` type import (E0433:L1229)
- [ ] `api_stitch_decompose::Arc::new(ws_connection_tracker)` - Missing `Arc` type import (E0433:L1230)
- [ ] `api_stitch_decompose::Arc::new(worker_ack_monitor)` - Missing `Arc` type import (E0433:L1231)
- [ ] `api_stitch_decompose::Arc::new(redaction_policy_state)` - Missing `Arc` type import (E0433:L1235)
- [ ] `api_stitch_decompose::Arc::new(stuck_detector)` - Missing `Arc` type import (E0433:L1238)
- [ ] `api_stitch_decompose::Arc::new(prompt_library)` - Missing `Arc` type import (E0433:L1241)
- [ ] `api_stitch_decompose::Arc::new(note_library)` - Missing `Arc` type import (E0433:L1242)
- [ ] `api_stitch_decompose::Arc::new(skill_library)` - Missing `Arc` type import (E0433:L1243)
- [ ] `api_stitch_decompose::Arc::new(script_library)` - Missing `Arc` type import (E0433:L1244)
- [ ] `api_stitch_decompose::Arc::new(config_status)` - Missing `Arc` type import (E0433:L1247)
- [ ] `api_stitch_decompose::std::time::Instant::now()` - Wrong Instant type, use tokio::time::Instant (E0308:L1204)
- [ ] `api_stitch_decompose::ProjectSupervisor::new()` - Missing 9 required arguments (E0061:L1213)
- [ ] `api_stitch_decompose::CostAggregator::new()` - Missing config_path argument (E0061:L1219)
- [ ] `api_stitch_decompose::UploadRegistry::new()` - Missing config argument (E0061:L1221)
- [ ] `api_stitch_decompose::ResolvedConfig::default()` - Default method not found (E0599:L1229)
- [ ] `api_stitch_decompose::RedactionPolicyState::default()` - Default method not found (E0599:L1236)
- [ ] `api_stitch_decompose::DaemonState` - Missing br_semaphore and br_semaphore_target_permits fields (E0063:L1202)

#### api_beads.rs (1 error)
- [ ] `api_beads::resolve_actor(None)` - Missing state argument (E0061:L1097)

#### api_preview.rs (1 error)
- [ ] `api_preview::PreviewRequest` - Missing attachments_count field (E0063:L621)

#### atomic_write.rs (1 error)
- [ ] `atomic_write::PathBuf::from()` - Missing PathBuf type import (E0433:L300)

#### capacity.rs (11 errors)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2456)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2502)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2572)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2773)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2850)
- [ ] `capacity::capacity_meter_config_new_accounts` - Missing 4 fields (E0063:L2912)
- [ ] `capacity::capacity_meter_config_new_gcp` - Missing 3 fields (E0063:L3057)
- [ ] `capacity::capacity_meter_config_new_gcp` - Missing 3 fields (E0063:L3110)
- [ ] `capacity::capacity_meter_config_new_opencode` - Missing 2 fields (E0063:L3202)
- [ ] `capacity::capacity_meter_config_new_opencode` - Missing 2 fields (E0063:L3226)
- [ ] `capacity::capacity_meter_config_new_opencode` - Missing 2 fields (E0063:L3266)

#### config_watcher.rs (15 errors)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L591)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L617)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L642)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L679)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L715)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L751)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L787)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L832)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L873)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L915)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L956)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L997)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L1038)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L1079)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L1122)
- [ ] `config_watcher::reload_config` - Missing agent_config_changed_tx argument (E0061:L1165)

#### dictated_notes.rs (1 error)
- [ ] `dictated_notes::DictatedNote` - Missing draft_id and synthesis_result fields (E0063:L776)

#### heartbeats.rs (2 errors)
- [ ] `heartbeats::prop_assert_eq!(computed_liveness)` - Returns Result but expected () (E0308:L935)
- [ ] `heartbeats::prop_assert_eq!(actual)` - Returns Result but expected () (E0308:L1089)

#### load_test.rs (1 error)
- [ ] `load_test::NeedleEvent::Fail` - Missing stash_sha field (E0063:L182)

#### redaction.rs (1 error)
- [ ] `redaction::SecretPattern::default_secret_patterns()` - Method not found (E0599:L498)

#### redaction_policy.rs (1 error)
- [ ] `redaction_policy::HoopConfig` - Missing embedding and redaction fields (E0063:L543)

#### syntax_highlight_stream.rs (8 errors)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L269)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L269)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)
- [ ] `syntax_highlight_stream::next().await` - Stream cannot be unpinned (E0277:L278)

### hoop-daemon (tests)

#### integration_harness.rs (2 errors)
- [ ] `integration_harness::DaemonHandle::_temp_dir` - Field renamed to temp_dir (E0609:L602)
- [ ] `integration_harness::Bead` - Missing workspace field (E0063:L269)

## Error Type Breakdown

| Error Code | Count | Description |
|------------|-------|-------------|
| E0433 | 23 | Cannot find type in scope (Arc, PathBuf imports) |
| E0061 | 18 | Function called with wrong number of arguments |
| E0063 | 17 | Missing fields in struct initializers |
| E0308 | 10 | Type mismatches |
| E0599 | 3 | Method or function not found |
| E0277 | 8 | Trait bound not satisfied (Unpin) |
| E0609 | 1 | No field on type |
| **Total** | **78** | |

## Categorized by Fix Type

### Import Errors (2 files, 24 errors)
- [ ] **api_stitch_decompose.rs**: Add `use std::sync::Arc;` (23 instances)
- [ ] **atomic_write.rs**: Add `use std::path::PathBuf;` (1 instance)

### Function Signature Mismatches (3 files, 18 errors)
- [ ] **config_watcher.rs**: Update all 16 `reload_config` calls to add 5th argument
- [ ] **api_beads.rs**: Add state parameter to `resolve_actor` call
- [ ] **api_stitch_decompose.rs**: Add 9 required arguments to `ProjectSupervisor::new()`

### Missing Struct Fields (7 files, 20 errors)
- [ ] **capacity.rs**: Add missing fields to 11 `CapacityMeterConfig` fixtures
- [ ] **api_preview.rs**: Add `attachments_count` to `PreviewRequest`
- [ ] **dictated_notes.rs**: Add `draft_id` and `synthesis_result` to `DictatedNote`
- [ ] **integration_harness.rs**: Add `workspace` field to `Bead`
- [ ] **redaction_policy.rs**: Add `embedding` and `redaction` to `HoopConfig`
- [ ] **load_test.rs**: Add `stash_sha` to `NeedleEvent::Fail`
- [ ] **api_stitch_decompose.rs**: Add `br_semaphore` fields to `DaemonState`

### Type Mismatches (2 files, 10 errors)
- [ ] **api_stitch_decompose.rs**: Use `tokio::time::Instant` instead of `std::time::Instant`
- [ ] **heartbeats.rs**: Fix `prop_assert_eq!` return type handling (2 instances)
- [ ] **syntax_highlight_stream.rs**: Add `pin!` macro for stream (8 instances)

### Missing Methods/Constructors (3 files, 4 errors)
- [ ] **api_stitch_decompose.rs**: Implement or replace `ResolvedConfig::default()`
- [ ] **api_stitch_decompose.rs**: Implement or replace `RedactionPolicyState::default()`
- [ ] **redaction.rs**: Implement or replace `SecretPattern::default_secret_patterns()`
- [ ] **api_stitch_decompose.rs**: Update `CostAggregator::new()` and `UploadRegistry::new()` calls

### Field Rename (1 file, 1 error)
- [ ] **integration_harness.rs**: Update `_temp_dir` to `temp_dir`

### Constructor Argument Fixes (2 files, 3 errors)
- [ ] **api_stitch_decompose.rs**: Update `CostAggregator::new()` call
- [ ] **api_stitch_decompose.rs**: Update `UploadRegistry::new()` call

## Next Steps

This checklist is ready for the next bead to begin systematic fixes. Recommended order:

1. **Fix imports first** (24 errors) - Quick wins, unblocks other fixes
2. **Fix function signatures** (18 errors) - Requires adding parameters
3. **Fix struct initializers** (20 errors) - Add missing fields
4. **Fix type mismatches** (10 errors) - Use correct types
5. **Fix missing methods** (4 errors) - Implement or find alternatives
6. **Fix field renames** (1 error) - Update to new field name
7. **Fix constructor calls** (3 errors) - Add required arguments

## Verification

After all fixes are applied, run:
```bash
nix-shell --run 'cargo test'
```

Expected result: Tests compile and run (may have runtime failures, but compilation should succeed).
