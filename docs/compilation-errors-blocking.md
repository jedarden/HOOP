# HOOP Blocking Compilation Errors Documentation

**Generated:** 2026-07-04  
**Total Blocking Errors:** 89  
**Status:** `cargo build` FAILS with compilation errors (not just warnings)

## Overview

The HOOP project currently has **89 compilation errors** that block the build. These are distinct from the 102 warnings (unused imports, unused variables) that do not prevent compilation. This document categorizes all blocking errors by severity and root cause.

---

## Priority 1: API Signature Mismatches (30 errors)

### Category: Function Parameter Count Mismatches

#### Error 1-15: `ConfigWatcher::reload_config` Missing 5th Parameter

**Files Affected:**
- `hoop-daemon/src/config_watcher.rs` (lines 591, 617, 642, 679, 715, 751, 787, 832, 873, 915, 956, 997, 1038, 1079, 1122, 1165)

**Error Message:**
```
error[E0061]: this function takes 5 arguments but 4 arguments were supplied
```

**Root Cause:**
The `ConfigWatcher::reload_config` function signature was updated to require a 5th parameter `agent_config_changed_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<AgentConfigChanged>>>>`, but all 16 call sites were not updated.

**Function Signature (line 304):**
```rust
async fn reload_config(
    config_path: &Path,
    event_tx: &broadcast::Sender<DaemonEvent>,
    shared_config: &Arc<RwLock<HoopConfig>>,
    cli_overrides: &CliOverrideConfig,
    agent_config_changed_tx: Arc<Mutex<Option<broadcast::Sender<AgentConfigChanged>>>>,  // MISSING
)
```

**Suggested Fix Approach:**
Add the missing parameter to all 16 call sites. The parameter should be passed through from the test harness or calling context:

```rust
// For each call site, add the 5th parameter:
ConfigWatcher::reload_config(
    &config_path,
    event_tx.clone(),
    shared_config.clone(),
    cli_overrides.clone(),
    agent_config_changed_tx.clone(),  // ADD THIS LINE
)
```

---

### Category: Struct Field Missing Errors (E0063)

#### Error 16: `Bead` Struct Missing `workspace` Field

**File:** `hoop-daemon/tests/integration_harness.rs:269`

**Error Message:**
```
error[E0063]: missing field `workspace` in initializer of `Bead`
```

**Root Cause:**
The `Bead` struct from `hoop_schema` now requires a `workspace` field, but test initialization code wasn't updated.

**Suggested Fix Approach:**
Add `workspace: "test-workspace".to_string()` to the `Bead` struct initialization.

---

#### Error 17: `PreviewRequest` Missing `attachments_count` Field

**File:** `hoop-daemon/src/api_preview.rs:621`

**Error Message:**
```
error[E0063]: missing field `attachments_count` in initializer of `api_preview::PreviewRequest`
```

**Root Cause:**
The `PreviewRequest` struct now requires an `attachments_count` field.

**Suggested Fix Approach:**
Add `attachments_count: 0` or compute the actual count from attachments.

---

#### Error 18: `DaemonState` Missing 2 Fields

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1203`

**Error Message:**
```
error[E0063]: missing fields `br_semaphore` and `br_semaphore_target_permits` in initializer of `DaemonState`
```

**Root Cause:**
The `DaemonState` struct now includes bead-rate-limiting semaphore fields that must be initialized.

**Suggested Fix Approach:**
```rust
let state = crate::DaemonState {
    // ... existing fields ...
    br_semaphore: Arc::new(Semaphore::new(10)),
    br_semaphore_target_permits: 10,
};
```

---

#### Error 19-26: `CapacityMeterConfig` Missing Multiple Fields

**File:** `hoop-daemon/src/capacity.rs` (lines 2457, 2503, 2573, 2774, 2851, 2913, 3058, 3111, 3203, 3227, 3267)

**Error Message:**
```
error[E0063]: missing fields `accounts_file`, `gcp_quota_config`, `gemini_dirs` and 1 other field in initializer of `capacity::CapacityMeterConfig`
```

**Root Cause:**
The `CapacityMeterConfig` struct was expanded with new required fields for GCP quota tracking and Gemini directory discovery. 11 test initialization sites weren't updated.

**Suggested Fix Approach:**
For each test site, add the missing fields:
```rust
let config = CapacityMeterConfig {
    // ... existing fields ...
    accounts_file: PathBuf::from("/tmp/test-accounts.json"),
    gcp_quota_config: None,
    gemini_dirs: Vec::new(),
    opencode_dirs: Vec::new(),
};
```

---

#### Error 27: `DictatedNote` Missing 2 Fields

**File:** `hoop-daemon/src/dictated_notes.rs:776`

**Error Message:**
```
error[E0063]: missing fields `draft_id` and `synthesis_result` in initializer of `dictated_notes::DictatedNote`
```

**Root Cause:**
The `DictatedNote` struct now tracks draft IDs and synthesis results for the voice note workflow.

**Suggested Fix Approach:**
```rust
&DictatedNote {
    // ... existing fields ...
    draft_id: None,
    synthesis_result: None,
}
```

---

#### Error 28: `HoopConfig` Missing 2 Fields

**File:** `hoop-daemon/src/redaction_policy.rs:546`

**Error Message:**
```
error[E0063]: missing fields `embedding` and `redaction` in initializer of `hoop_schema::HoopConfig`
```

**Root Cause:**
The `HoopConfig` struct now includes embedding and redaction configuration sections.

**Suggested Fix Approach:**
```rust
let config = HoopConfig {
    // ... existing fields ...
    embedding: None,
    redaction: None,
};
```

---

#### Error 29: `NeedleEvent::Fail` Missing `stash_sha` Field

**File:** `hoop-daemon/src/load_test.rs:182`

**Error Message:**
```
error[E0063]: missing field `stash_sha` in initializer of `events::NeedleEvent`
```

**Root Cause:**
The `NeedleEvent::Fail` variant now requires a `stash_sha` field for Git commit tracking.

**Suggested Fix Approach:**
```rust
NeedleEvent::Fail {
    // ... existing fields ...
    stash_sha: "abc123".to_string(),
}
```

---

## Priority 2: Type Mismatches (25 errors)

### Category: Instant Type Mismatches

#### Error 30: `std::time::Instant` vs `tokio::time::Instant`

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1205`

**Error Message:**
```
error[E0308]: mismatched types
expected `tokio::time::Instant`, found `std::time::Instant`
```

**Root Cause:**
The code uses `std::time::Instant::now()` but the field expects `tokio::time::Instant`.

**Suggested Fix Approach:**
```rust
started_at: tokio::time::Instant::now(),
// OR convert:
started_at: std::time::Instant::now().into(),
```

---

### Category: WebSocket Message Type Changes

#### Error 31-36: WebSocket Text Message Utf8Bytes Requirement

**File:** `hoop-daemon/tests/integration_harness.rs` (lines 862, 1105, 1202, 1214, 1222)

**Error Message:**
```
error[E0308]: mismatched types
expected `Utf8Bytes`, found `String`
```

**Root Cause:**
The `tokio-tungstenite` crate upgraded to 0.26.x, which changed `Message::Text` to require `Utf8Bytes` instead of `String`.

**Suggested Fix Approach:**
Add `.into()` to convert String to Utf8Bytes:
```rust
.send(tokio_tungstenite::tungstenite::Message::Text(
    subscribe_msg.to_string().into(),  // ADD .into()
))
```

---

### Category: Constructor Result Type Mismatches

#### Error 37-41: Fallible Constructors Used in Arc::new

**Files:**
- `hoop-daemon/src/api_stitch_decompose.rs:1220` (CostAggregator)
- `hoop-daemon/src/api_stitch_decompose.rs:1222` (UploadRegistry)
- `hoop-daemon/src/api_stitch_decompose.rs:1232` (WorkerAckMonitor)

**Error Message:**
```
error[E0308]: mismatched types
expected `StructType`, found `Result<StructType, Error>`
```

**Root Cause:**
These constructors now return `Result<T, Error>` but are being called directly in `Arc::new()` without unwrapping.

**Suggested Fix Approach:**
```rust
// BEFORE (incorrect):
cost_aggregator: Arc::new(std::sync::RwLock::new(crate::cost::CostAggregator::new())),

// AFTER (correct):
cost_aggregator: Arc::new(std::sync::RwLock::new(
    crate::cost::CostAggregator::new(PathBuf::from("/tmp/cost.yaml"))
        .expect("Failed to create CostAggregator")
)),
```

---

### Category: Return Type Mismatches

#### Error 42-43: Property Test Return Type Issues

**File:** `hoop-daemon/src/heartbeats.rs` (lines 935, 1089)

**Error Message:**
```
error[E0308]: mismatched types
expected `()`, found `Result<(), _>`
```

**Root Cause:**
Property test macros use `prop_assert_eq!` which return `Result<(), _>`, but the test function expects `()`.

**Suggested Fix Approach:**
```rust
// The issue is that prop_assert_eq! returns Result but fn expects ()
// Change test signature to return Result:
#[test]
fn prop_test_name() -> prop::test_runner::TestResult {  // CHANGED FROM ()
    // ... test code ...
    Ok(())  // EXPLICITLY RETURN OK
}
```

---

## Priority 3: Missing Trait Implementations (3 errors)

### Category: PartialEq Required

#### Error 44-46: `RiskSeverity` Missing PartialEq

**File:** `hoop-daemon/src/risk_patterns.rs` (lines 662, 666, 670)

**Error Message:**
```
error[E0369]: binary operation `==` cannot be applied to type `risk_patterns::RiskSeverity`
```

**Root Cause:**
The `RiskSeverity` enum is used in `assert_eq!` macros but doesn't implement `PartialEq`.

**Suggested Fix Approach:**
Add `#[derive(PartialEq)]` to the enum:
```rust
#[derive(PartialEq)]  // ADD THIS
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

---

## Priority 4: Missing or Changed Methods (10 errors)

### Category: Constructor Signature Changes

#### Error 47: `ProjectSupervisor::new` Now Requires 9 Arguments

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1214`

**Error Message:**
```
error[E0061]: this function takes 9 arguments but 0 arguments were supplied
```

**Root Cause:**
The `ProjectSupervisor::new` constructor was refactored to require dependencies explicitly instead of creating them internally.

**Required Arguments:**
1. `bead_tx: broadcast::Sender<BeadEvent>`
2. `session_tx: broadcast::Sender<SessionEvent>`
3. `worker_registry: Arc<WorkerRegistry>`
4. `beads: Arc<RwLock<Vec<Bead>>>`
5. `shutdown: Arc<ShutdownCoordinator>`
6. `cost_aggregator: Arc<RwLock<CostAggregator>>`
7. `vector_index: Arc<RwLock<VectorIndex>>`
8. `scripts_dir: PathBuf`
9. `stuck_detector: Arc<Mutex<StuckDetector>>`

**Suggested Fix Approach:**
Refactor the test to create all dependencies first, then pass them to `ProjectSupervisor::new()`.

---

#### Error 48: `CostAggregator::new` Now Requires PathBuf

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1220`

**Error Message:**
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
```

**Root Cause:**
`CostAggregator::new` now requires a config path.

**Suggested Fix Approach:**
```rust
CostAggregator::new(PathBuf::from("/tmp/cost-config.yaml")).expect("Failed to create CostAggregator")
```

---

#### Error 49: `UploadRegistry::new` Now Requires UploadConfig

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1222`

**Error Message:**
```
error[E0061]: this function takes 1 argument but 0 arguments were supplied
```

**Root Cause:**
`UploadRegistry::new` now requires an `UploadConfig` parameter.

**Suggested Fix Approach:**
```rust
let upload_config = uploads::UploadConfig {
    max_size_bytes: 10_000_000,
    allowed_mime_types: vec!["image/png".to_string(), "image/jpeg".to_string()],
    storage_dir: PathBuf::from("/tmp/uploads"),
};
UploadRegistry::new(upload_config).expect("Failed to create UploadRegistry")
```

---

### Category: Default Implementation Missing

#### Error 50: `ResolvedConfig` Missing Default Trait

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1230`

**Error Message:**
```
error[E0599]: no associated function or constant named `default` found for struct `ResolvedConfig`
```

**Root Cause:**
The test code tries to use `ResolvedConfig::default()` but the struct doesn't implement `Default`.

**Suggested Fix Approach:**
Either implement `Default` for `ResolvedConfig` or construct it explicitly:
```rust
// Option 1: Add derive to struct
#[derive(Default)]
pub struct ResolvedConfig {
    // ...
}

// Option 2: Construct explicitly
ResolvedConfig {
    projects: std::collections::HashMap::new(),
    // ... other fields with defaults
}
```

---

#### Error 51: `RedactionPolicyState` Missing Default Trait

**File:** `hoop-daemon/src/api_stitch_decompose.rs:1237`

**Error Message:**
```
error[E0599]: no associated function or constant named `default` found for struct `redaction_policy::RedactionPolicyState`
```

**Root Cause:**
Similar to above, `RedactionPolicyState` doesn't implement `Default`.

**Suggested Fix Approach:**
Use `RedactionPolicyState::new()` instead, which requires:
- `global_config: &HoopConfig`
- `projects_registry: ProjectsRegistry`

---

#### Error 52: `SecretPattern::default_secret_patterns` Missing

**File:** `hoop-daemon/src/redaction.rs:498`

**Error Message:**
```
error[E0599]: no associated function or constant named `default_secret_patterns` found
```

**Root Cause:**
The `SecretPattern::default_secret_patterns()` method was renamed or removed.

**Suggested Fix Approach:**
Check if the method was renamed to `SecretPattern::defaults()` or construct patterns explicitly.

---

### Category: Field Access Changes

#### Error 53: `DaemonHandle._temp_dir` Field Name Changed

**File:** `hoop-daemon/tests/integration_harness.rs:602`

**Error Message:**
```
error[E0609]: no field `_temp_dir` on type `DaemonHandle`
```

**Root Cause:**
The private field `_temp_dir` was renamed to `temp_dir` (made public).

**Suggested Fix Approach:**
```rust
// BEFORE:
handle._temp_dir

// AFTER:
handle.temp_dir
```

---

## Priority 5: Missing Dependencies (1 error)

#### Error 54: Missing `rand` Crate

**File:** `hoop-daemon/src/integration_harness.rs:192`

**Error Message:**
```
error[E0433]: cannot find module or crate `rand` in this scope
```

**Root Cause:**
The test uses `rand::random()` but the `rand` crate is not in `Cargo.toml` dependencies.

**Suggested Fix Approach:**
Add to `hoop-daemon/Cargo.toml`:
```toml
[dev-dependencies]
rand = "0.8"
```

---

## Summary by Category

| Category | Error Count | Severity |
|----------|-------------|----------|
| Function Parameter Mismatches | 16 | HIGH |
| Struct Field Missing (E0063) | 14 | HIGH |
| Type Mismatches (E0308) | 25 | HIGH |
| Missing Trait Implementations | 3 | MEDIUM |
| Constructor Signature Changes | 7 | HIGH |
| Default Trait Missing | 3 | MEDIUM |
| Field Access Changes | 1 | LOW |
| Missing Dependencies | 1 | LOW |
| Other API Changes | 19 | MEDIUM |

---

## Recommended Fix Strategy

### Phase 1: API Signature Updates (HIGH PRIORITY)
1. Fix all 16 `ConfigWatcher::reload_config` call sites (add 5th parameter)
2. Update all `CapacityMeterConfig` initializations (11 sites)
3. Fix `ProjectSupervisor::new` constructor call (1 site, refactor test)
4. Fix `CostAggregator::new` and `UploadRegistry::new` calls (2 sites)

### Phase 2: Struct Field Updates (HIGH PRIORITY)
1. Add `workspace` to `Bead` initialization
2. Add `attachments_count` to `PreviewRequest`
3. Add `br_semaphore` and `br_semaphore_target_permits` to `DaemonState`
4. Add `draft_id` and `synthesis_result` to `DictatedNote`
5. Add `embedding` and `redaction` to `HoopConfig`
6. Add `stash_sha` to `NeedleEvent::Fail`

### Phase 3: Type Mismatches (HIGH PRIORITY)
1. Fix `std::time::Instant` vs `tokio::time::Instant` (1 site)
2. Fix WebSocket Utf8Bytes conversions (5 sites)
3. Fix Result unwrapping in Arc::new (3 sites)
4. Fix property test return types (2 sites)

### Phase 4: Trait Implementations (MEDIUM PRIORITY)
1. Add `#[derive(PartialEq)]` to `RiskSeverity`

### Phase 5: Missing Dependencies (LOW PRIORITY)
1. Add `rand` to dev-dependencies

---

## Automation Opportunities

1. **Struct field updates** can be automated with `cargo fix` or regex-based find-replace for repetitive patterns
2. **WebSocket Utf8Bytes** changes are mechanical (add `.into()` to all `.to_string()` calls in Message::Text)
3. **ConfigWatcher parameter** additions are identical across 16 sites (scriptable)

---

## Notes

- All 102 warnings are separate from these 89 errors
- Warnings are cosmetic (unused imports, unused variables) and do not block compilation
- The errors stem from a major refactoring that updated struct definitions and constructor signatures
- Many errors are in test code (`tests/` and `integration_harness.rs`)
- Some errors indicate incomplete implementation (missing `Default` traits, missing methods)

---

## Verification Checklist

After fixes are applied, verify with:

```bash
# Should compile cleanly
nix-shell --run 'cargo build'

# Should have no errors (warnings OK)
nix-shell --run 'cargo clippy -- -D warnings'

# Should pass all tests
nix-shell --run 'cargo test'
```

**Success Criteria:**
- [ ] `cargo build` completes with exit code 0
- [ ] `cargo clippy -- -D warnings` has zero errors
- [ ] Test suite compiles and runs
