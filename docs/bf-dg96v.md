# Risk Patterns Test Compilation Blockers

## Overview
This document catalogs all compilation errors preventing `risk_patterns::tests` from running, categorized by module and fix complexity.

## Direct risk_patterns Module Errors (4 errors)

### Category: Missing Trait Derivations
**Effort: Low** (5 minutes)

All errors are in `/home/coding/HOOP/hoop-daemon/src/risk_patterns.rs` at lines 662, 666, 670, 674.

**Error Pattern:**
```
error[E0369]: binary operation `==` cannot be applied to type `risk_patterns::RiskSeverity`
```

**Root Cause:** The `RiskSeverity` enum (line 35) lacks `#[derive(PartialEq)]`.

**Current Definition:**
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

**Fix:** Add `PartialEq` to the derive macro:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

**Impact:** This single fix resolves all 4 errors in the risk_patterns module and unblocks all 17 tests.

---

## Test Infrastructure Errors (89 total errors)

### Category: Integration Harness
**Module:** `hoop-daemon/tests/integration_harness.rs`

| Error | Line | Type | Fix Effort |
|-------|------|------|------------|
| E0609 | 602 | Field access: `_temp_dir` → `temp_dir` | Low |
| E0063 | 269 | Missing field: `workspace` in `Bead` | Medium |
| E0308 | 862, 1105, 1202, 1214, 1222 | Type mismatch: `String` → `Utf8Bytes` (add `.into()`) | Low |

### Category: API Stitch Decompose
**Module:** `hoop-daemon/src/api_stitch_decompose.rs`

| Error | Line | Type | Fix Effort |
|-------|------|------|------------|
| E0063 | 621 | Missing field: `attachments_count` in `PreviewRequest` | Low |
| E0308 | 1205 | Type mismatch: `std::time::Instant` → `tokio::time::Instant` | Low |
| E0061 | 1214 | Wrong arg count: `ProjectSupervisor::new()` takes 9 args, got 0 | High |
| E0061 | 1220 | Wrong arg count: `CostAggregator::new()` takes 1 arg, got 0 | Medium |
| E0061 | 1222 | Wrong arg count: `UploadRegistry::new()` takes 1 arg, got 0 | Medium |
| E0599 | 1230 | Missing method: `ResolvedConfig::default()` | Medium |
| E0308 | 1232 | Type mismatch: Expected `WorkerAckMonitor`, got `Result` | Low |
| E0599 | 1237 | Missing method: `RedactionPolicyState::default()` | Medium |
| E0063 | 1203 | Missing fields: `br_semaphore`, `br_semaphore_target_permits` in `DaemonState` | Medium |

### Category: Config Watcher
**Module:** `hoop-daemon/src/config_watcher.rs`

| Error Pattern | Count | Type | Fix Effort |
|---------------|-------|------|------------|
| E0061 | 13 | Wrong arg count: `reload_config()` takes 5 args, got 4 | Medium |

All at lines: 591, 617, 642, 679, 715, 751, 787, 832, 873, 915, 956, 997, 1038, 1079, 1122, 1165

**Missing Argument:** `agent_config_changed_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<AgentConfigChanged>>>>`

### Category: Capacity Meter Config
**Module:** `hoop-daemon/src/capacity.rs`

| Error | Lines | Type | Fix Effort |
|-------|-------|------|------------|
| E0063 | 2457, 2503, 2573, 2774, 2851, 2913 | Missing fields: `accounts_file`, `gcp_quota_config`, `gemini_dirs`, `opencode_dirs` | High |
| E0063 | 3058, 3111 | Missing fields: `accounts_file`, `gcp_quota_config`, `opencode_dirs` | High |
| E0063 | 3203, 3227, 3267 | Missing fields: `accounts_file`, `opencode_dirs` | Medium |

### Category: Other Modules
**Module:** Various

| Error | Module | Line | Type | Fix Effort |
|-------|--------|------|------|------------|
| E0063 | dictated_notes | 776 | Missing fields: `draft_id`, `synthesis_result` | Medium |
| E0308 | heartbeats | 935, 1089 | Type mismatch: Expected `()`, got `Result` | Low |
| E0433 | multiple | - | Missing module: `rand` | Low |
| E0063 | events | - | Missing field: `stash_sha` in `NeedleEvent` | Low |
| E0599 | config_resolver | - | Missing method: `SecretPattern::default_secret_patterns()` | Medium |
| E0063 | hoop_schema | - | Missing fields: `embedding`, `redaction` in `HoopConfig` | High |
| E0063 | syntax_highlight_stream | - | Missing field: `a` in `Color` (4 instances) | Low |
| E0063 | net_diff | 547, 552 | Missing field: `bead_id` in `CommitEntry` | Low |

### Category: Async Unpin Errors
**Module:** `hoop-daemon/src/syntax_highlight_stream.rs`

| Error Pattern | Count | Type | Fix Effort |
|---------------|-------|------|------------|
| E0277 | 16 | Unpin trait not implemented for async blocks | High |

All at lines: 301, 308 (×4), 315 (×4), 322 (×4)

**Issue:** Async blocks in stream cannot be unpinned, requires pinning or Box::pin

---

## Dependency Chain Analysis

### Direct Blockers for risk_patterns tests
```
risk_patterns tests
    ↓ (blocked by)
RiskSeverity missing PartialEq
    ↓ (fixes)
Add #[derive(PartialEq)] to RiskSeverity
    ↓ (enables)
All 17 risk_patterns tests can compile and run
```

### Full Test Suite Blockers
```
cargo test --package hoop-daemon --lib --tests risk_patterns
    ↓ (blocked by)
1. RiskSeverity errors (4) → BLOCKS risk_patterns tests specifically
2. Integration harness errors (7) → BLOCKS integration tests
3. API stitch decompose errors (9) → BLOCKS decompose tests
4. Config watcher errors (13) → BLOCKS config reload tests
5. Capacity config errors (11) → BLOCKS capacity tests
6. Other module errors (35) → BLOCKS other unit/integration tests
```

---

## Prioritized Fix List

### Priority 1: Unblock risk_patterns Tests
**Total Effort: 5 minutes**

1. **Add `PartialEq` to `RiskSeverity`** (`risk_patterns.rs:35`)
   - Change: `#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]`
   - Impact: Unblocks all 17 risk_patterns tests

### Priority 2: Quick Wins (Low-hanging fruit)
**Total Effort: ~30 minutes**

1. **Fix WebSocket message types** (integration_harness.rs)
   - Add `.into()` to 5 String → Utf8Bytes conversions
   
2. **Fix temp_dir field access** (integration_harness.rs:602)
   - Change `handle._temp_dir` → `handle.temp_dir`

3. **fix Color struct initialization** (syntax_highlight_stream.rs)
   - Add missing `a` field to 4 Color literals

4. **Fix CommitEntry initialization** (net_diff.rs)
   - Add missing `bead_id` field

5. **Add rand dependency** (Cargo.toml if missing)
   - Or remove unused `rand` imports

### Priority 3: Medium Complexity Fixes
**Total Effort: ~2-3 hours**

1. **Fix ConfigWatcher::reload_config calls** (13 sites)
   - Add missing `agent_config_changed_tx` parameter

2. **Fix missing struct fields** (multiple modules)
   - Add missing fields to struct initializations
   
3. **Fix Result types** (api_stitch_decompose.rs)
   - Add `.expect()` or proper error handling

### Priority 4: High Complexity Fixes
**Total Effort: ~4-6 hours**

1. **Fix ProjectSupervisor::new() calls** (api_stitch_decompose.rs:1214)
   - Requires providing 9 parameters or creating a builder

2. **Fix CapacityMeterConfig initialization** (capacity.rs, 11 sites)
   - Requires providing multiple complex config fields

3. **Fix async Unpin issues** (syntax_highlight_stream.rs, 16 sites)
   - Requires pinning async blocks or restructuring stream

---

## Testing Strategy After Fixes

### Phase 1: risk_patterns Unit Tests
```bash
# After fixing RiskSeverity
cargo test --package hoop-daemon --lib risk_patterns::tests
```

### Phase 2: Full Lib Tests
```bash
# After fixing all lib errors
cargo test --package hoop-daemon --lib
```

### Phase 3: Integration Tests
```bash
# After fixing integration harness
cargo test --package hoop-daemon --tests
```

---

## Estimated Effort Summary

| Category | Error Count | Estimated Time |
|----------|-------------|----------------|
| Direct risk_patterns fixes | 4 | 5 minutes |
| Quick wins | ~20 | 30 minutes |
| Medium complexity | ~25 | 2-3 hours |
| High complexity | ~40 | 4-6 hours |
| **TOTAL** | **89** | **~7-10 hours** |

---

## Key Files to Modify

### Risk Patterns Module
- `/home/coding/HOOP/hoop-daemon/src/risk_patterns.rs` - Add PartialEq derive

### Test Infrastructure
- `/home/coding/HOOP/hoop-daemon/tests/integration_harness.rs` - Fix struct field accesses
- `/home/coding/HOOP/hoop-daemon/src/api_stitch_decompose.rs` - Fix constructor calls
- `/home/coding/HOOP/hoop-daemon/src/config_watcher.rs` - Add missing parameter
- `/home/coding/HOOP/hoop-daemon/src/capacity.rs` - Fix config initialization

### Other Modules
- `/home/coding/HOOP/hoop-daemon/src/syntax_highlight_stream.rs` - Fix async pinning
- `/home/coding/HOOP/hoop-daemon/src/net_diff.rs` - Add missing struct field
- Various - Add missing struct fields and fix type mismatches

---

## Conclusion

**Critical Finding:** The risk_patterns module itself has only 4 compilation errors, all stemming from a single missing trait derive. Adding `#[derive(PartialEq)]` to `RiskSeverity` will unblock all 17 risk_patterns tests immediately.

**Broader Issue:** The full test suite has 89 compilation errors across multiple modules, indicating significant API drift and missing constructor parameters following recent refactors. These should be fixed systematically, starting with the highest-value tests.

**Recommendation:** Fix the RiskSeverity enum first to unblock risk_patterns tests (5 minutes), then prioritize fixes based on which test suites provide the most value.
