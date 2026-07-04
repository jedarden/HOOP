# Compilation Error Catalog - risk_patterns Dependency Chain

**Bead:** bf-2c7x0
**Date:** 2026-07-04
**Task:** Catalog all compilation errors in risk_patterns dependency chain
**Total Errors:** 95 compilation errors + 32 warnings

## Executive Summary

Running `cargo test --lib` on hoop-daemon revealed **95 compilation errors** across 15 files. Errors fall into 8 primary categories:

1. Missing imports (E0433): 23 errors
2. Missing function arguments (E0061): 17 errors  
3. Missing struct fields (E0063): 20 errors
4. Type mismatches (E0308): 3 errors
5. Missing associated functions (E0599): 3 errors
6. Unpin trait violations (E0277): 26 errors
7. Other errors: 3 errors

## Affected Modules

- `api_stitch_decompose.rs` - 48 errors (most impacted)
- `capacity.rs` - 11 errors
- `config_watcher.rs` - 13 errors
- `syntax_highlight_stream.rs` - 26 errors
- `atomic_write.rs` - 1 error
- `api_beads.rs` - 1 error
- `api_preview.rs` - 1 error
- `heartbeats.rs` - 2 errors
- `load_test.rs` - 1 error
- `redaction.rs` - 1 error
- `redaction_policy.rs` - 1 error
- `dictated_notes.rs` - 1 error
- `net_diff.rs` - 2 errors
- `agent_session.rs` - 2 warnings
- `reflection_detector.rs` - 1 warning
- `lib.rs` - 3 warnings
- Various files - 26+ warnings

## Detailed Error Catalog

### Category 1: Missing Imports (E0433) - 23 errors

#### Missing `Arc` type in api_stitch_decompose.rs (22 errors)

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Lines:** 1197, 1198, 1205, 1209, 1212, 1213, 1214, 1215, 1219, 1221, 1222, 1223, 1229, 1230, 1231, 1235, 1238, 1241, 1242, 1243, 1244, 1247

**Error:**
```
error[E0433]: cannot find type `Arc` in this scope
    --> hoop-daemon/src/api_stitch_decompose.rs:1197:30
     |
1197 |         let identity_cache = Arc::new(crate::identity::IdentityCache::new());
     |                              ^^^ use of undeclared type `Arc`
```

**Fix:** Add `use std::sync::Arc;` to imports

#### Missing `PathBuf` type in atomic_write.rs (1 error)

**File:** `hoop-daemon/src/atomic_write.rs`
**Line:** 300

**Error:**
```
error[E0433]: cannot find type `PathBuf` in this scope
    --> hoop-daemon/src/atomic_write.rs:300:20
     |
300 |         let dest = PathBuf::from("file.txt");
     |                    ^^^^^^^ use of undeclared type `PathBuf`
```

**Fix:** Add `use std::path::PathBuf;` to imports

---

### Category 2: Missing Function Arguments (E0061) - 17 errors

#### resolve_actor missing second argument

**File:** `hoop-daemon/src/api_beads.rs`
**Line:** 1097

**Error:**
```
error[E0061]: this function takes 2 arguments but 1 argument was supplied
    --> hoop-daemon/src/api_beads.rs:1097:21
     |
1097 |         let actor = resolve_actor(None);
     |                     ^^^^^^^^^^^^^------ argument #2 of type `&DaemonState` is missing
```

**Signature:** `fn resolve_actor(remote_addr: Option<SocketAddr>, state: &crate::DaemonState) -> String`

**Fix:** Add state parameter: `resolve_actor(None, &state)`

#### ConfigWatcher::reload_config missing 5th argument (13 errors)

**File:** `hoop-daemon/src/config_watcher.rs`
**Lines:** 591, 617, 642, 679, 715, 751, 787, 832, 873, 915, 956, 997, 1038, 1079, 1122, 1165

**Error:**
```
error[E0061]: this function takes 5 arguments but 4 arguments were supplied
   --> hoop-daemon/src/config_watcher.rs:591:9
    |
591 |           ConfigWatcher::reload_config(
    |  _________^^^^^^^^^^^^^^^^^^^^^^^^^^^^-
592 | |             &config_path,
593 | |             event_tx.clone(),
594 | |             shared_config.clone(),
595 | |             cli_overrides.clone(),
596 | |         )
    | |_________- argument #5 of type `std::sync::Arc<tokio::sync::Mutex<std::option::Option<tokio::sync::broadcast::Sender<AgentConfigChanged>>>>` is missing
```

**Signature:** `async fn reload_config(config_path, event_tx, shared_config, cli_overrides, agent_config_changed_tx)`

**Fix:** Add agent_config_changed_tx parameter

#### ProjectSupervisor::new missing 9 arguments

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1213

**Error:**
```
error[E0061]: this function takes 9 arguments but 0 arguments were supplied
    --> hoop-daemon/src/api_stitch_decompose.rs:1213:34
     |
1213 |             supervisor: Arc::new(crate::supervisor::ProjectSupervisor::new()),
     |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^-- multiple arguments are missing
```

**Signature:** 
```rust
pub fn new(
    bead_tx: broadcast::Sender<BeadEvent>,
    session_tx: broadcast::Sender<SessionEvent>,
    worker_registry: Arc<crate::ws::WorkerRegistry>,
    beads: Arc<std::sync::RwLock<Vec<Bead>>>,
    shutdown: Arc<crate::shutdown::ShutdownCoordinator>,
    cost_aggregator: Arc<std::sync::RwLock<CostAggregator>>,
    vector_index: Arc<std::sync::RwLock<crate::vector_index::VectorIndex>>,
    scripts_dir: PathBuf,
    stuck_detector: Arc<std::sync::Mutex<crate::stuck_detector::StuckDetector>>,
)
```

**Fix:** Provide all 9 required arguments

#### CostAggregator::new missing 1 argument

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1219

**Error:**
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
    --> hoop-daemon/src/api_stitch_decompose.rs:1219:62
     |
1219 |             cost_aggregator: Arc::new(std::sync::RwLock::new(crate::cost::CostAggregator::new())),
     |                                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^-- argument #1 of type `std::path::PathBuf` is missing
```

**Signature:** `pub fn new(config_path: PathBuf) -> Result<Self>`

**Fix:** Add config_path parameter

#### UploadRegistry::new missing 1 argument

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1221

**Error:**
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
    --> hoop-daemon/src/api_stitch_decompose.rs:1221:39
     |
1221 |             upload_registry: Arc::new(crate::uploads::UploadRegistry::new()),
     |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^-- argument #1 of type `uploads::UploadConfig` is missing
```

**Signature:** `pub fn new(config: UploadConfig) -> Result<Self>`

**Fix:** Add UploadConfig parameter

---

### Category 3: Missing Struct Fields (E0063) - 20 errors

#### PreviewRequest missing attachments_count

**File:** `hoop-daemon/src/api_preview.rs`
**Line:** 621

**Error:**
```
error[E0063]: missing field `attachments_count` in initializer of `api_preview::PreviewRequest`
   --> hoop-daemon/src/api_preview.rs:621:22
    |
621 |         let params = PreviewRequest {
    |                      ^^^^^^^^^^^^^^ missing `attachments_count`
```

**Fix:** Add `attachments_count: 0` (or appropriate value)

#### DaemonState missing br_semaphore fields

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1202

**Error:**
```
error[E0063]: missing fields `br_semaphore` and `br_semaphore_target_permits` in initializer of `DaemonState`
   --> hoop-daemon/src/api_stitch_decompose.rs:1202:21
    |
1202 |         let state = crate::DaemonState {
    |                     ^^^^^^^^^^^^^^^^^^ missing `br_semaphore` and `br_semaphore_target_permits`
```

**Fix:** Add both missing fields

#### CapacityMeterConfig missing fields (11 errors)

**File:** `hoop-daemon/src/capacity.rs`
**Lines:** 2456, 2502, 2572, 2773, 2850, 2912, 3057, 3110, 3202, 3226, 3266

**Errors:**
- Lines 2456, 2502, 2572, 2773, 2850, 2912: Missing `accounts_file`, `gcp_quota_config`, `gemini_dirs`, and 1 other field
- Lines 3057, 3110: Missing `accounts_file`, `gcp_quota_config`, and `opencode_dirs`
- Lines 3202, 3226, 3266: Missing `accounts_file` and `opencode_dirs`

**Fix:** Add appropriate fields based on context

#### DictatedNote missing draft_id and synthesis_result

**File:** `hoop-daemon/src/dictated_notes.rs`
**Line:** 776

**Error:**
```
error[E0063]: missing fields `draft_id` and `synthesis_result` in initializer of `dictated_notes::DictatedNote`
   --> hoop-daemon/src/dictated_notes.rs:776:18
    |
776 |                 &DictatedNote {
    |                  ^^^^^^^^^^^^ missing `draft_id` and `synthesis_result`
```

**Fix:** Add both missing fields

#### NeedleEvent::Fail missing stash_sha

**File:** `hoop-daemon/src/load_test.rs`
**Line:** 182

**Error:**
```
error[E0063]: missing field `stash_sha` in initializer of `events::NeedleEvent`
   --> hoop-daemon/src/load_test.rs:182:29
    |
182 |                 events.push(NeedleEvent::Fail {
    |                             ^^^^^^^^^^^^^^^^^ missing `stash_sha`
```

**Fix:** Add `stash_sha` field

#### HoopConfig missing embedding and redaction

**File:** `hoop-daemon/src/redaction_policy.rs`
**Line:** 543

**Error:**
```
error[E0063]: missing fields `embedding` and `redaction` in initializer of `hoop_schema::HoopConfig`
   --> hoop-daemon/src/redaction_policy.rs:543:22
    |
543 |         let config = HoopConfig {
    |                      ^^^^^^^^^^ missing `embedding` and `redaction`
```

**Fix:** Add both missing fields

#### CommitEntry missing bead_id (2 errors)

**File:** `hoop-daemon/src/net_diff.rs`
**Lines:** 547, 552

**Error:**
```
error[E0063]: missing field `bead_id` in initializer of `net_diff::CommitEntry`
   --> hoop-daemon/src/net_diff.rs:547:13
    |
547 |             CommitEntry {
    |             ^^^^^^^^^^^ missing `bead_id`
```

**Fix:** Add `bead_id` field

---

### Category 4: Type Mismatches (E0308) - 3 errors

#### std::time::Instant vs tokio::time::Instant

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1204

**Error:**
```
error[E0308]: mismatched types
   --> hoop-daemon/src/api_stitch_decompose.rs:1204:25
    |
1204 |             started_at: std::time::Instant::now(),
    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `tokio::time::Instant`, found `std::time::Instant`
```

**Fix:** Use `tokio::time::Instant::now()` instead

#### Property test return type issues (2 errors)

**File:** `hoop-daemon/src/heartbeats.rs`
**Lines:** 935, 1089

**Error:**
```
error[E0308]: mismatched types
   --> hoop-daemon/src/heartbeats.rs:935:13
    |
935 |             Ok(())
    |             ^^^^^^ expected `()`, found `Result<(), _>`
```

**Fix:** Property test macros need unit type, not Result

---

### Category 5: Missing Associated Functions (E0599) - 3 errors

#### ResolvedConfig::default not found

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1229

**Error:**
```
error[E0599]: no associated function or constant named `default` found for struct `ResolvedConfig` in the current scope
   --> hoop-daemon/src/api_stitch_decompose.rs:1229:79
    |
1229 |             resolved_config: Arc::new(crate::config_resolver::ResolvedConfig::default()),
     |                                                                               ^^^^^^^ associated function or constant not found in `ResolvedConfig`
```

**Fix:** Implement Default trait or use appropriate constructor

#### RedactionPolicyState::default not found

**File:** `hoop-daemon/src/api_stitch_decompose.rs`
**Line:** 1236

**Error:**
```
error[E0599]: no associated function or constant named `default` found for struct `redaction_policy::RedactionPolicyState` in the current scope
   --> hoop-daemon/src/api_stitch_decompose.rs:1236:64
    |
1236 |                 crate::redaction_policy::RedactionPolicyState::default(),
     |                                                                ^^^^^^^ associated function or constant not found in `redaction_policy::RedactionPolicyState`
```

**Fix:** Use `RedactionPolicyState::new()` instead

#### SecretPattern::default_secret_patterns not found

**File:** `hoop-daemon/src/redaction.rs`
**Line:** 498

**Error:**
```
error[E0599]: no associated function or constant named `default_secret_patterns` found for struct `config_resolver::SecretPattern` in the current scope
   --> hoop-daemon/src/redaction.rs:498:47
    |
498 |         let default_patterns = SecretPattern::default_secret_patterns();
     |                                               ^^^^^^^^^^^^^^^^^^^^^^^ associated function or constant not found in `config_resolver::SecretPattern`
```

**Fix:** Implement or use correct function name

---

### Category 6: Unpin Trait Violations (E0277) - 26 errors

**File:** `hoop-daemon/src/syntax_highlight_stream.rs`
**Lines:** 269, 269, 278, 278, 286, 286, 286, 286, 301, 301, 301, 301, 308, 308, 308, 308, 315, 315, 315, 315, 322, 322, 322, 322

**Error Pattern:**
```
error[E0277]: `{async block@hoop-daemon/src/syntax_highlight_stream.rs:163:18: 163:28}` cannot be unpinned
   --> hoop-daemon/src/syntax_highlight_stream.rs:269:28
    |
269 |         let first = stream.next().await.unwrap();
    |                            ^^^^ unsatisfied trait bound
```

**Root Cause:** Async blocks in stream are not Unpin, but StreamExt::next requires Unpin

**Fix Options:**
1. Use `pin!` macro to pin the stream
2. Use `Box::pin` to create a pinned box
3. Restructure stream to avoid async blocks that aren't Unpin

---

### Category 7: Warnings - 32 warnings

#### Unused Imports (18 warnings)
- `json` in prompt_substitute.rs:15
- `utoipa::ToSchema` in 13 files (agent_session.rs, api_stitch_decompose.rs, api_stitch_replay.rs, config_resolver.rs, fleet.rs, content_blocks.rs, predictor.rs, uploads.rs, ws.rs, api_ui_state.rs, screen_capture.rs, fix_patterns.rs, files.rs, stitch_decompose.rs, syntax_highlight.rs, api_prompts.rs)
- `std::fs::File`, `tempfile::TempDir` in hoop-mcp/src/skills.rs
- `std::path::PathBuf` in net_diff.rs:419
- `AtomicBool`, `Ordering` in agent_session.rs:2260
- `crate::tag_join` in sessions.rs:3562

#### Dead Code (4 warnings)
- Function `openapi_router` in lib.rs:1277
- Function `load_hoop_config` in lib.rs:3796
- Function `check_and_emit_capacity_alert` in lib.rs:4063
- Struct `QuotaLimit` in capacity.rs:60

#### Unused Fields (3 warnings)
- Field `session_id` in capacity.rs:358
- Field `session_subpath` in capacity.rs:526
- Field `rpm_limit` in capacity.rs:55
- Field `subpath` in sessions.rs:557

#### Unused Constants (3 warnings)
- `MAX_UNASSIGNED_SESSIONS` in sessions.rs:763
- `MIN_SAMPLES_FOR_PREDICTION` in stitch_percentile_index.rs:68
- `STITCH_CLOSED_THRESHOLD_SECONDS` in stitch_percentile_index.rs:72

#### Unused Variables (4 warnings)
- `tmp` in atomic_write.rs:461
- `words` in audio_redaction.rs:456
- `start` in load_test.rs:471
- `client_id` in load_test.rs:610
- `http_elapsed` in load_test.rs:678
- `out` in pdf_sanitize.rs:503, 512, 519, 539
- `paths` in sessions.rs:3701
- `home` in config_backup.rs:147
- `has_high_entropy` in secrets_scanner.rs:528

#### Other Warnings
- Unused mut in load_test.rs:485
- Private interface in reflection_detector.rs:88 (PatternCategory more private than DetectedPattern::category)

---

## Summary Statistics

### By File
| File | Errors |
|------|--------|
| api_stitch_decompose.rs | 48 |
| syntax_highlight_stream.rs | 26 |
| config_watcher.rs | 13 |
| capacity.rs | 11 |
| net_diff.rs | 2 |
| heartbeats.rs | 2 |
| atomic_write.rs | 1 |
| api_beads.rs | 1 |
| api_preview.rs | 1 |
| dictated_notes.rs | 1 |
| load_test.rs | 1 |
| redaction.rs | 1 |
| redaction_policy.rs | 1 |

### By Error Type
| Error Type | Count |
|------------|-------|
| E0433 (missing type) | 23 |
| E0061 (wrong argument count) | 17 |
| E0063 (missing field) | 20 |
| E0277 (Unpin trait) | 26 |
| E0308 (type mismatch) | 3 |
| E0599 (missing function) | 3 |
| E0560 (struct field visibility) | 3 |

---

## Priority Recommendations

### High Priority (Blocks Compilation)
1. Fix all missing imports (23 errors) - trivial fixes
2. Fix struct field mismatches (20 errors) - requires understanding correct field values
3. Fix function argument mismatches (17 errors) - requires understanding correct parameters
4. Fix Unpin trait violations (26 errors) - requires async/stream expertise

### Medium Priority
5. Fix type mismatches (3 errors)
6. Fix missing associated functions (3 errors)

### Low Priority (Can be deferred)
7. Address warnings (32 warnings) - doesn't block compilation

---

## Next Steps

1. Address missing imports first (quickest wins)
2. Fix Unpin violations in syntax_highlight_stream.rs (most complex)
3. Update struct initializers with missing fields
4. Fix function call signatures
5. Implement missing Default traits or use correct constructors
