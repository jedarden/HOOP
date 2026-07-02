# Prioritized Fix Plan for HOOP Compilation Issues

**Generated:** 2026-07-02  
**Bead:** bf-fwva2  
**Total Issues:** 88 (14 blocking errors + 74 non-blocking warnings)  
**Phase 1 CI Gate Status:** ❌ BLOCKED

---

## Executive Summary

This plan prioritizes fixing all documented compilation issues in the HOOP project. Issues are organized by dependency order, impact, and effort. The primary goal is to unblock the Phase 1 CI gate (bead `bf-5mpcl`).

**Quick Stats:**
- **P0 (Critical):** 4 structs need `#[derive(ToSchema)]` → 14 compilation errors
- **P1 (High):** 74 warnings need cleanup → 0 blocking but required for CI gate
- **Estimated Time:** P0: 30 minutes, P1: 1-2 hours

---

## Phase 1: CRITICAL - Unblocking Compilation (P0)

**Status:** ❌ NOT STARTED  
**Blocker:** Missing `ToSchema` trait implementations  
**Impact:** Blocks `cargo build`, `cargo test`, and all Phase 1 work  
**Dependencies:** None  
**Estimated Time:** 30 minutes

### Issue Group P0.1: OpenAPI Schema Trait Bounds (14 errors)

All four structs are missing `#[derive(ToSchema)]` and are used in OpenAPI endpoint documentation via `utoipa`. Each struct generates 2 errors (one for `ToSchema`, one for `PartialSchema`).

#### Fix P0.1.1: `ListJobsQuery` (2 errors)
**File:** `hoop-daemon/src/api_transcription.rs:19`  
**Referenced in:** `hoop-daemon/src/openapi.rs:500`  
**Severity:** BLOCKING  
**Fix:** Add `#[derive(ToSchema)]` to struct definition

```rust
// Current (line ~19)
pub struct ListJobsQuery {
    pub workspace: String,
    pub status: Option<String>,
    pub limit: Option<u32>,
}

// Fixed
#[derive(ToSchema)]
pub struct ListJobsQuery {
    pub workspace: String,
    pub status: Option<String>,
    pub limit: Option<u32>,
}
```

#### Fix P0.1.2: `CreateScreenCaptureRequest` (2 errors)
**File:** `hoop-daemon/src/api_screen_capture.rs:34`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:84`  
**Severity:** BLOCKING  
**Fix:** Add `#[derive(ToSchema)]` to struct definition

```rust
// Current (line ~34)
struct CreateScreenCaptureRequest {
    pub project: String,
    pub stitch_id: String,
    pub format: CaptureFormat,
}

// Fixed
#[derive(ToSchema)]
struct CreateScreenCaptureRequest {
    pub project: String,
    pub stitch_id: String,
    pub format: CaptureFormat,
}
```

#### Fix P0.1.3: `StartStreamingUploadRequest` (2 errors)
**File:** `hoop-daemon/src/api_screen_capture.rs:352`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:366`  
**Severity:** BLOCKING  
**Fix:** Add `#[derive(ToSchema)]` to struct definition

```rust
// Current (line ~352)
pub struct StartStreamingUploadRequest {
    pub project: String,
    pub stitch_id: String,
    pub format: CaptureFormat,
}

// Fixed
#[derive(ToSchema)]
pub struct StartStreamingUploadRequest {
    pub project: String,
    pub stitch_id: String,
    pub format: CaptureFormat,
}
```

#### Fix P0.1.4: `CompleteStreamingUploadRequest` (2 errors)
**File:** `hoop-daemon/src/api_screen_capture.rs:469`  
**Referenced in:** `hoop-daemon/src/api_screen_capture.rs:484`  
**Severity:** BLOCKING  
**Fix:** Add `#[derive(ToSchema)]` to struct definition

```rust
// Current (line ~469)
pub struct CompleteStreamingUploadRequest {
    pub project: String,
    pub stitch_id: String,
    pub upload_id: String,
}

// Fixed
#[derive(ToSchema)]
pub struct CompleteStreamingUploadRequest {
    pub project: String,
    pub stitch_id: String,
    pub upload_id: String,
}
```

### Verification Step P0.2: Build Verification

After applying all P0 fixes:

```bash
# Verify compilation succeeds
nix-shell --run 'cargo build'

# Expected: 0 compilation errors
# If errors remain, they are likely from other structs not documented in recent analysis
```

**Success Criteria:** 
- ✅ `cargo build` completes with exit code 0
- ✅ No compilation errors in output
- ✅ Phase 1 CI gate no longer blocked by trait bound errors

---

## Phase 2: HIGH - Code Quality Cleanup (P1)

**Status:** ❌ NOT STARTED  
**Blocker:** None (proceeds after P0)  
**Impact:** Required for `cargo clippy -- -D warnings` to pass (Phase 1 CI gate requirement)  
**Dependencies:** P0 must be complete first  
**Estimated Time:** 1-2 hours

### Issue Group P1.1: Unused Imports (36-38 warnings)

**Severity:** NON-BLOCKING but required for CI gate  
**Auto-Fixable:** YES (via `cargo clippy --fix`)  
**Files Affected:** 23 files

**Locations:**
| File | Unused Imports | Count |
|------|----------------|-------|
| accounts_config.rs | PathBuf, warn | 2 |
| api_bead_files.rs | State, Connection, params, Deserialize | 4 |
| api_pattern_mutations.rs | get | 1 |
| api_stitch_decompose.rs | std::sync::Arc | 1 |
| api_stitch_replay.rs | ReplayOptions | 1 |
| api_unassigned.rs | ParsedSessionKind | 1 |
| api_skills.rs | RecommendedWatcher | 1 |
| capacity.rs | StdDuration, AccountsOpenCodeLimits | 2 |
| content_blocks.rs | chrono::Utc | 1 |
| api_presence.rs | HashMap | 1 |
| api_tour_project.rs | get | 1 |
| migrations.rs | Serialize | 1 |
| stitch_reconstruction.rs | anyhow, HashMap | 2 |
| stuck_detector.rs | anyhow::Result | 1 |
| prompt_substitute.rs | anyhow, bail, json | 3 |
| api_prompts.rs | SubstitutionContext | 1 |
| config_backup.rs | warn | 1 |
| cross_project_propagation.rs | SimilarStitch, DateTime | 2 |
| api_fix_patterns.rs | delete, put | 2 |
| api_screen_capture.rs | self | 1 |
| screen_capture.rs | Path | 1 |
| saturation_detector.rs | Deserialize, Serialize | 2 |
| observer.rs | log_rotation, TcpStream | 2 |
| lib.rs | AgentConfigChanged | 1 |

**Fix:**
```bash
# Auto-fix most unused imports
nix-shell --run 'cargo clippy --fix --allow-dirty -p hoop-daemon'
```

### Issue Group P1.2: Unused Variables (30-33 warnings)

**Severity:** NON-BLOCKING but required for CI gate  
**Auto-Fixable:** PARTIAL (some need manual prefixing with `_`)  
**Hotspot Files:** auth.rs, api_scripts.rs, api_skills.rs, lib.rs, cross_project_propagation.rs

**High-Density Locations:**
| File | Variables | Lines |
|------|-----------|-------|
| auth.rs | remote_addr, required_role (×2) | 329, 338 |
| api_scripts.rs | start, timed_out (×2) | 312, 361, 368 |
| api_skills.rs | start, timed_out (×2) | 284, 344, 350 |
| lib.rs | abs_path, project, synthesis_callback, semaphore_ref | 975, 2413, 2415, 3077 |
| cross_project_propagation.rs | created_by, conn, sim, source_labels | 220, 451, 469, 475 |

**Fix Pattern:**
```rust
// Option 1: Prefix with underscore (if intentionally unused for future use)
let _remote_addr = extract_remote_addr();
let _required_role = Role::Admin;

// Option 2: Remove entirely (if truly dead code)
// Delete the line completely

// Option 3: Use the variable (if it was meant to be used)
log::info!("Remote address: {}", remote_addr);
```

### Issue Group P1.3: Unnecessary `mut` Keywords (3-8 warnings)

**Severity:** NON-BLOCKING but required for CI gate  
**Auto-Fixable:** NO (manual review required)  
**Files Affected:** api_tour_project.rs, api_fix_patterns.rs, lib.rs

**Locations:**
| File | Line | Variable |
|------|------|----------|
| api_tour_project.rs | 240 | conn |
| api_fix_patterns.rs | 454 | conn |
| lib.rs | 3446 | shutdown_rx |

**Fix Pattern:**
```rust
// Before
let mut conn = get_connection();

// After (if never mutated)
let conn = get_connection();
```

### Verification Step P1.4: Clippy Clean

After applying all P1 fixes:

```bash
# Run clippy with warnings as errors
nix-shell --run 'cargo clippy -- -D warnings'

# Expected: 0 warnings
# If warnings remain, manually address remaining issues
```

**Success Criteria:**
- ✅ `cargo clippy -- -D warnings` completes with exit code 0
- ✅ No warnings in output
- ✅ Phase 1 CI gate clippy requirement satisfied

---

## Phase 3: MEDIUM - Additional Verification (P2)

**Status:** ❌ NOT STARTED  
**Blocker:** None  
**Impact:** Verify no additional issues from older analysis  
**Dependencies:** P0 and P1 must be complete  
**Estimated Time:** 30 minutes

### Issue Group P2.1: Legacy Struct Checks

**Note:** The older `bf-xibss` analysis (2025-01-16) documented additional structs that may need `ToSchema` derives. These were NOT present in more recent analyses (bf-305tm, bf-2ncvr, bf-bbhnf), suggesting either:
1. They were already fixed
2. They are not actually used in OpenAPI paths
3. The analysis was outdated

**Additional structs from bf-xibss to verify:**
- `api_agent::SwitchRequest` (api_agent.rs:127)
- `api_agent::TurnRequest` (api_agent.rs:194)
- `api_agent::TurnAttachment` (api_agent.rs:186)
- `cross_project_propagation::SiblingProject` (cross_project_propagation.rs:23)
- `api_reflection_ledger::ApproveProposalRequest` (api_reflection_ledger.rs:42)
- `api_reflection_ledger::RejectProposalRequest` (api_reflection_ledger.rs:59)
- `api_scripts::ScriptRunRequest` (api_scripts.rs:162)
- `api_tour_project::EnableTourRequest` (api_tour_project.rs:34)

**Verification Step:**
```bash
# After P0 and P1 are complete, run a fresh build
nix-shell --run 'cargo build 2>&1 | grep -E "error\[E0277\].*ToSchema"'

# If errors appear for these structs, they need the same fix pattern as P0
```

**Success Criteria:**
- ✅ All structs referenced in OpenAPI paths have `#[derive(ToSchema)]`
- ✅ No new compilation errors appear after P0 fixes

---

## Phase 4: LOW - Test Verification (P3)

**Status:** ❌ NOT STARTED  
**Blocker:** None  
**Impact:** Verify Phase 1 CI gate fully passes  
**Dependencies:** P0, P1, P2 must be complete  
**Estimated Time:** 30 minutes

### Verification Step P3.1: Full Test Suite

After all compilation and clippy issues are resolved:

```bash
# Run full test suite
nix-shell --run 'cargo test'

# Expected: All tests pass
# This validates the fixes don't break existing functionality
```

### Verification Step P3.2: Phase 1 CI Gate Validation

Validate all Phase 1 exit criteria from the plan:

```bash
# 1. Build succeeds
nix-shell --run 'cargo build'
# Expected: exit code 0

# 2. Tests pass
nix-shell --run 'cargo test'
# Expected: All tests pass

# 3. Clippy clean
nix-shell --run 'cargo clippy -- -D warnings'
# Expected: exit code 0

# 4. Verify hoop status --json works (if implemented)
nix-shell --run 'cargo run --bin hoop -- status --json | jq .'
# Expected: Valid JSON output
```

**Success Criteria:**
- ✅ All Phase 1 CI gate requirements pass
- ✅ Bead `bf-5mpcl` can be closed
- ✅ Phase 2 work can begin (per phase sequence lock)

---

## Summary of Fix Order

### Execution Sequence (Strict Order):

1. **P0.1.1:** Add `#[derive(ToSchema)]` to `ListJobsQuery` (api_transcription.rs:19)
2. **P0.1.2:** Add `#[derive(ToSchema)]` to `CreateScreenCaptureRequest` (api_screen_capture.rs:34)
3. **P0.1.3:** Add `#[derive(ToSchema)]` to `StartStreamingUploadRequest` (api_screen_capture.rs:352)
4. **P0.1.4:** Add `#[derive(ToSchema)]` to `CompleteStreamingUploadRequest` (api_screen_capture.rs:469)
5. **P0.2:** Verify `cargo build` succeeds
6. **P1.1:** Run `cargo clippy --fix` for unused imports
7. **P1.2:** Manually fix unused variables (prefix with `_` or remove)
8. **P1.3:** Remove unnecessary `mut` keywords
9. **P1.4:** Verify `cargo clippy -- -D warnings` clean
10. **P2.1:** Verify no legacy struct issues remain
11. **P3.1:** Run full test suite
12. **P3.2:** Validate Phase 1 CI gate exit criteria

---

## Rationale for Priority Levels

### Why P0 First?
- **Blocks everything:** Without fixing these 4 structs, `cargo build` fails, making tests unreachable
- **Fast to fix:** 4 simple derive macro additions, ~30 minutes
- **Clear verification:** Single binary pass/fail (build succeeds or fails)

### Why P1 Second?
- **Required for CI:** Phase 1 gate requires `cargo clippy -- -D warnings` to pass
- **Mostly auto-fixable:** 36 of 74 warnings are auto-fixed by clippy
- **High-leverage:** Clears most code quality debt in one batch

### Why P2 Third?
- **Verification only:** Confirms older analysis is either obsolete or already fixed
- **Low risk:** If issues exist, they follow the same pattern as P0 (simple derive adds)

### Why P3 Last?
- **Gate validation:** Ensures all fixes integrate correctly
- **Prevents regressions:** Full test suite confirms no functionality broken

---

## Dependencies Between Issues

```
P0 (Trait bounds) 
  └─ Must be complete before any other phase
     └─ Blocks cargo build, making all other work impossible

P1 (Code quality)
  └─ Depends on: P0 complete
  └─ Enables: cargo clippy to pass

P2 (Legacy verification)
  └─ Depends on: P0 and P1 complete
  └─ Ensures: No additional trait bound issues

P3 (Test validation)
  └─ Depends on: P0, P1, P2 complete
  └─ Validates: Phase 1 CI gate ready to close
```

---

## Effort vs Impact Matrix

| Phase | Issues | Effort | Impact | Blocker? |
|-------|--------|--------|--------|----------|
| P0 | 14 errors (4 structs) | 30 min | CRITICAL | YES |
| P1 | 74 warnings | 1-2 hrs | HIGH | NO (but required for CI) |
| P2 | 0-8 errors | 30 min | MEDIUM | NO |
| P3 | Test validation | 30 min | HIGH | NO (depends on P0-P2) |

**Total Estimated Time:** 2.5-3.5 hours

---

## Risk Assessment

### Low Risk Fixes
- Adding `#[derive(ToSchema)]` - purely additive, no behavior change
- Removing unused imports - reduces clutter, no functional impact
- Removing unused variables - reduces clutter, no functional impact
- Removing unnecessary `mut` - makes code more honest, no behavior change

### Medium Risk Fixes
- Fixing unused variables with `_` prefix - needs review to ensure truly unused
- Removing `mut` keywords - needs review to ensure no future mutation planned

### Mitigation Strategy
1. Commit after each phase completion (P0, P1, P2, P3)
2. Run tests after P0 to verify no behavior change
3. Manual review of all unused variable fixes before applying

---

## Acceptance Criteria

### Phase 0 (P0) Complete:
- ✅ `cargo build` succeeds with 0 compilation errors
- ✅ All 4 structs have `#[derive(ToSchema)]`
- ✅ Git commit created with P0 fixes

### Phase 1 (P1) Complete:
- ✅ `cargo clippy -- -D warnings` succeeds with 0 warnings
- ✅ All unused imports removed
- ✅ All unused variables addressed
- ✅ All unnecessary `mut` keywords removed
- ✅ Git commit created with P1 fixes

### Phase 2 (P2) Complete:
- ✅ No additional trait bound errors appear
- ✅ Any legacy structs fixed if needed
- ✅ Git commit created with P2 fixes (if any)

### Phase 3 (P3) Complete:
- ✅ `cargo test` passes all tests
- ✅ Phase 1 CI gate exit criteria validated
- ✅ Bead `bf-5mpcl` ready to close
- ✅ Git commit created documenting validation

---

## References

### Documentation Sources:
- `notes/bf-305tm.md` - Blocking issues with locations (8 errors)
- `notes/bf-2ncvr.md` - Structured compilation error documentation (14 errors + 74 warnings)
- `notes/bf-bbhnf.md` - Blocking errors with locations (14 errors)
- `notes/bf-5t13a.md` - Compilation error logs location guide
- `notes/bf-19zug.md` - Error categorization by type and severity
- `notes/bf-jmb87.md` - Original debug build analysis
- `notes/bf-xibss.md` - Clippy warnings (older, possibly outdated)

### Related Beads:
- `bf-5mpcl` - Phase 1 CI gate (blocked by these issues)
- `bf-89p2v` - Original error extraction
- Genesis bead `hoop-ttb` - Overall project tracking

### Plan Reference:
- `docs/plan/plan.md` - Phase 1 deliverables and exit criteria

---

**Bead:** bf-fwva2  
**Status:** PLAN COMPLETE  
**Next Action:** Execute P0 fixes (4 structs)
