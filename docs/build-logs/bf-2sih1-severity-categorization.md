# HOOP Build Issues - Severity Categorization

**Build Status:** ✅ PASSES (0 errors, 102 warnings)

This document categorizes all compilation issues by blocking severity and priority.

## Summary

- **Blocking Issues:** 0 (build compiles successfully)
- **Non-Blocking Issues:** 102 warnings
- **Severity Distribution:**
  - Critical: 0
  - High: 0
  - Medium: ~47 (unused imports, dead code, visibility)
  - Low: ~55 (unused variables, mut warnings, naming conventions)

---

## Non-Blocking Issues (All Warnings)

### Medium Severity (~47 warnings)

#### 1. Unused Imports (30+ warnings)
**Impact:** Code compiles but has unused imports. Can be cleaned up with `cargo fix`.

**Files affected:**
- `hoop-daemon/src/accounts_config.rs:27` - `PathBuf`
- `hoop-daemon/src/accounts_config.rs:28` - `warn`
- `hoop-daemon/src/api_bead_files.rs:11:17` - `State`, `Connection`, `params`, `Deserialize`
- `hoop-daemon/src/api_pattern_mutations.rs:14` - `get`
- `hoop-daemon/src/api_stitch_decompose.rs:30` - `Arc`
- `hoop-daemon/src/api_stitch_replay.rs:8` - `ReplayOptions`
- `hoop-daemon/src/api_unassigned.rs:23` - `ParsedSessionKind`
- `hoop-daemon/src/api_skills.rs:39` - `RecommendedWatcher`
- `hoop-daemon/src/atomic_write.rs:42` - `PathBuf`
- `hoop-daemon/src/capacity.rs:25` - `StdDuration`, `AccountsOpenCodeLimits`
- `hoop-daemon/src/content_blocks.rs:7` - `Utc`
- `hoop-daemon/src/api_presence.rs:20` - `HashMap`
- `hoop-daemon/src/api_tour_project.rs:12` - `get`
- `hoop-daemon/src/migrations.rs:51` - `Serialize`
- `hoop-daemon/src/stitch_reconstruction.rs:19` - `anyhow`, `HashMap`
- `hoop-daemon/src/stuck_detector.rs:20` - `Result`
- `hoop-daemon/src/prompt_substitute.rs:13:15` - `anyhow`, `bail`, `json`
- `hoop-daemon/src/api_prompts.rs:45` - `SubstitutionContext`
- `hoop-daemon/src/config_backup.rs:14` - `warn`
- `hoop-daemon/src/cross_project_propagation.rs:15:17` - `SimilarStitch`, `DateTime`
- `hoop-daemon/src/api_fix_patterns.rs:16` - `delete`, `put`
- `hoop-daemon/src/api_screen_capture.rs:12` - `self`
- `hoop-daemon/src/screen_capture.rs:23` - `Path`
- `hoop-daemon/src/saturation_detector.rs:17` - `Deserialize`, `Serialize`
- `hoop-daemon/src/observer.rs:8` - `log_rotation`, `TcpStream`
- `hoop-cli/src/config.rs:11` - `env`
- `hoop-cli/src/patterns.rs:17` - `Deserialize`, `Serialize`
- `hoop-cli/src/skills.rs:8:12` - `ArgGroup`, `Args`, `Parser`, `Subcommand`, `Write`
- `hoop-cli/src/main.rs:21` - `Serialize`

**Fix:** Run `cargo fix --lib -p hoop-daemon --bin hoop` to auto-fix ~80 of these warnings.

#### 2. Dead Code / Unused Functions (9 warnings)
**Impact:** Unused code increases binary size and maintenance burden.

**Items:**
- `hoop-daemon/src/lib.rs:1277` - `openapi_router()` - never used
- `hoop-daemon/src/lib.rs:3799` - `load_hoop_config()` - never used
- `hoop-daemon/src/lib.rs:4076` - `check_and_emit_capacity_alert()` - never used
- `hoop-daemon/src/capacity.rs:473` - `get_opencode_limits()` - never used
- `hoop-cli/src/projects.rs:391` - `validate_workspace()` - never used
- Multiple unused structs: `QuotaLimit` (capacity.rs:61)
- Multiple unused constants: `MAX_UNASSIGNED_SESSIONS`, `MIN_SAMPLES_FOR_PREDICTION`, `STITCH_CLOSED_THRESHOLD_SECONDS`
- Multiple unused struct fields: `ParsedPrompt.session_id`, `GeminiSessionPath.subpath`, `GeminiQuotaLimits.rpm_limit`, etc.

**Fix:** Remove or annotate with `#[allow(dead_code)]` if kept for future use.

#### 3. Type Visibility Issue (1 warning)
**Impact:** Public field exposes private type, causing potential API confusion.

```
hoop-daemon/src/reflection_detector.rs:88:5
warning: type `PatternCategory` is more private than the item `DetectedPattern::category`
```

**Fix:** Either make `PatternCategory` public or make `DetectedPattern::category` private.

### Low Severity (~55 warnings)

#### 1. Unused Variables (25+ warnings)
**Impact:** Code compiles but has unused variables.

**Variables:**
- `start` timing variables (8 instances) - meant for instrumentation but never used
- `conn` connections (2 instances) - opened but never read
- `timed_out` flags (2 instances) - assigned but never checked
- Various unused parameters: `remote_addr`, `required_role`, `link_kind`, `schedule`, `overlap_policy`, etc.

**Fix:** Prefix with underscore (`_start`, `_conn`) or remove.

#### 2. Unnecessary `mut` Declarations (8 warnings)
**Impact:** Code compiles but declares mutable when immutable would suffice.

**Locations:**
- `hoop-daemon/src/api_tour_project.rs:240` - `conn`
- `hoop-daemon/src/api_fix_patterns.rs:454` - `conn`
- `hoop-daemon/src/lib.rs:3446` - `shutdown_rx`
- `hoop-daemon/src/capacity.rs:593:596` - `gemini_dirs`, `opencode_dirs`
- `hoop-daemon/src/cross_project_propagation.rs:468:476` - `shared_files`, `shared_labels`
- `hoop-daemon/src/fix_patterns.rs:83:277` - `conn` (2 instances)

**Fix:** Remove `mut` keyword.

#### 3. Naming Convention Issues (1 warning)
**Impact:** Code compiles but violates Rust naming conventions.

```
hoop-cli/src/init.rs:36:5
warning: structure field `DNSName` should have a snake case name
```

**Fix:** Rename to `dnsname`.

#### 4. Lifetime Syntax Confusion (1 warning)
**Impact:** Code compiles but has confusing lifetime annotation.

```
hoop-daemon/src/api_pattern_mutations.rs:566
warning: eliding a lifetime that's named elsewhere is confusing
```

**Fix:** Add explicit lifetime to return type: `-> &'a [&'a dyn rusqlite::ToSql]`

---

## Priority Recommendations

### High Priority (Address before Phase 1 completion)
1. **Fix visibility issue in reflection_detector.rs** - Exposes private type publicly
2. **Fix lifetime syntax in api_pattern_mutations.rs** - Confusing signature

### Medium Priority (Code quality cleanup)
1. Run `cargo fix` to auto-resolve unused imports (~80 warnings)
2. Remove or document intentionally-unused functions (~9 warnings)
3. Clean up unused variables with underscore prefix (~25 warnings)

### Low Priority (Polish)
1. Remove unnecessary `mut` declarations (~8 warnings)
2. Fix naming convention violation (~1 warning)

---

## Phase 1 Impact Assessment

**Does this block Phase 1 completion?** ❌ NO

The build compiles successfully. These are warnings only and do not prevent:
- `cargo test` execution
- Binary functionality
- API endpoint operation

**Recommendation:** Address as code quality cleanup, but not blocking for Phase 1 CI gate. Can be fixed incrementally.
