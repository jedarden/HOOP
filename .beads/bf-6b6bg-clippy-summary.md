# Clippy Warnings Summary - bead bf-6b6bg

**Date:** 2026-08-06
**Task:** Check current clippy warnings and project structure
**Result:** Project builds successfully; 88 clippy errors found

## Build Status

✅ **hoop-daemon builds successfully** - `cargo check --workspace` passes with only warnings
✅ **Target directory exists** - `.beads/` directory confirmed at `/home/coding/HOOP/.beads/`

## Clippy Results

**Total Errors:** 88 (reported by clippy at line 1008)
**Command:** `cargo clippy --workspace -- -D warnings`

### Error Categories

#### 1. Disallowed Methods (28 errors) - **CRITICAL**
These use unsafe file I/O that should be replaced with crash-safe atomic writes:

**`std::fs::write` (19 instances):**
- `hoop-daemon/src/attachment_sync.rs:86`
- `hoop-daemon/src/attachments.rs:613`
- `hoop-daemon/src/backup_pipeline.rs:561`
- `hoop-daemon/src/metrics.rs:1336`
- `hoop-daemon/src/parse_jsonl_safe.rs:236`
- `hoop-daemon/src/projects.rs:113`
- `hoop-daemon/src/template_library.rs:424`
- `hoop-daemon/src/uploads.rs:132, 446, 470, 516, 540`
- `hoop-daemon/src/api_screen_capture.rs:149, 164, 214`
- `hoop-daemon/src/screen_capture.rs:353, 418, 452, 494`

**`std::fs::File::create` (9 instances):**
- `hoop-daemon/src/atomic_write.rs:97, 188`
- `hoop-daemon/src/log_rotation.rs:110, 117, 145`
- `hoop-daemon/src/uploads.rs:190`
- `hoop-daemon/src/screen_capture.rs:358`

**Fix:** Replace with `atomic_write::atomic_write_file()` or `atomic_write::atomic_write_file_str()`

---

#### 2. Dead Code (13 errors)
Functions, fields, constants, and structs that are never used:

- `hoop-daemon/src/lib.rs:1307` - `openapi_router` function
- `hoop-daemon/src/lib.rs:3826` - `load_hoop_config` function
- `hoop-daemon/src/lib.rs:4093` - `check_and_emit_capacity_alert` function
- `hoop-daemon/src/capacity.rs:358` - `session_id` field in `ParsedPrompt`
- `hoop-daemon/src/capacity.rs:472` - `get_opencode_limits` function
- `hoop-daemon/src/capacity.rs:526` - `session_subpath` field in `GeminiAccountPaths`
- `hoop-daemon/src/capacity.rs:55` - `rpm_limit` field in `GeminiQuotaLimits`
- `hoop-daemon/src/capacity.rs:60` - `QuotaLimit` struct
- `hoop-daemon/src/sessions.rs:557` - `subpath` field in `GeminiSessionPath`
- `hoop-daemon/src/sessions.rs:763` - `MAX_UNASSIGNED_SESSIONS` constant
- `hoop-daemon/src/stitch_percentile_index.rs:72` - `STITCH_CLOSED_THRESHOLD_SECONDS` constant

**Fix:** Remove unused code or add `#[allow(dead_code)]` if kept for future use

---

#### 3. Style/Clarity Improvements (30+ errors)

**Manual clamp (2 instances):**
- `hoop-daemon/src/api_cost_per_stitch.rs:109` - Use `.clamp(1, 180)` instead of `.min(180).max(1)`
- `hoop-daemon/src/api_timeline.rs:76` - Use `.clamp(1, 168)` instead of `.min(168).max(1)`

**Unnecessary sort_by (5 instances):**
- `hoop-daemon/src/api_conversations.rs:327`
- `hoop-daemon/src/api_onboarding.rs:228`
- `hoop-daemon/src/api_stitch_read.rs:470`
- `hoop-daemon/src/api_unassigned.rs:320`
- `hoop-daemon/src/lib.rs:1219`
- Fix: Use `.sort_by_key(|b| std::cmp::Reverse(b.field))`

**Lines filter_map_ok (4 instances):**
- `hoop-daemon/src/api_scripts.rs:344, 352`
- `hoop-daemon/src/api_skills.rs:325, 333`
- `hoop-daemon/src/lib.rs:4033`
- Fix: Use `.map_while(Result::ok)` instead of `.flatten()`

**Manual strip prefix (5 instances):**
- `hoop-daemon/src/api_blame.rs:228, 237`
- `hoop-daemon/src/api_diff.rs:169, 178`
- Fix: Use `.strip_prefix()` instead of manual slicing

**Explicit counter loop (3 instances):**
- `hoop-daemon/src/capacity.rs:1613, 1772`
- `hoop-daemon/src/sessions.rs:1155`
- Fix: Use `.enumerate()` instead of manual counter

**Other style issues:**
- Filter_next: `hoop-daemon/src/api_conversations.rs:144` - Use `.rfind()`
- Unnecessary unwrap: `hoop-daemon/src/capacity.rs:603, 604` - Use `if let`
- If same then else: `hoop-daemon/src/config_resolver.rs:371` - Merge identical blocks
- Map identity: `hoop-daemon/src/config_resolver.rs:1544, 2396` - Remove `.map(|v| v)`
- Ptr arg: `hoop-daemon/src/pdf_sanitize.rs:184, 222, 259, 279, 311` - Use `&mut [_]` instead of `&mut Vec`
- Doc overindented: `hoop-daemon/src/supervisor.rs:1091` - Fix list indentation

---

#### 4. Function/Struct Signature Issues (9 errors)

**Too many arguments (5 instances):**
- `hoop-daemon/src/config_resolver.rs:679, 1678` - Functions with 9 arguments (max 7)
- `hoop-daemon/src/fleet.rs:645` - `create_stitch_with_audit` with 12 arguments
- `hoop-daemon/src/fleet.rs:5217, 5322` - Functions with 8 arguments

**Large enum variant (1 instance):**
- `hoop-daemon/src/config_watcher.rs:40` - `ConfigEvent` has 2160-byte variant vs 136-byte (should Box large fields)

**Type complexity (5 instances):**
- `hoop-daemon/src/cost.rs:499` - Complex HashMap type
- `hoop-daemon/src/metrics.rs:397, 449, 516, 658` - Complex metric types
- `hoop-daemon/src/stitch_percentile_index.rs:582` - Complex Vec type

**Fix:** Consider refactoring into structs or type aliases

---

#### 5. Missing Trait Implementations (3 errors)

**Should implement trait (2 instances):**
- `hoop-daemon/src/embedding_service.rs:68` - `from_str` method should implement `std::str::FromStr`
- `hoop-daemon/src/stuck_detector.rs:66` - `from_str` method should implement `std::str::FromStr`

**Missing Default implementation (1 instance):**
- `hoop-daemon/src/fleet_notifications.rs:127` - `FleetNotificationRing::new()` should have `Default` impl

---

#### 6. Missing Method (1 error)

**len_without_is_empty:**
- `hoop-daemon/src/identity.rs:123` - `IdentityCache` has `len()` but no `is_empty()` method

---

#### 7. Private Interface Violations (3 errors)

- `hoop-daemon/src/pattern_query_evaluator.rs:88` - `QueryExpr` type is more private than `parse_query`
- `hoop-daemon/src/pattern_query_evaluator.rs:258` - `QueryExpr` type is more private than `evaluate_query`
- `hoop-daemon/src/reflection_detector.rs:88` - `PatternCategory` type is more private than `DetectedPattern::category`

**Fix:** Make types at least as visible as their consuming functions

---

## Next Steps

The project is ready for the filtered clippy run to exclude utoipa warnings as specified in the task. The full clippy output has been saved to:
- `.beads/clippy-output-full.txt` - Complete 1008-line clippy output
- `.beads/bf-6b6bg-clippy-summary.md` - This summary

**Status:** ✅ Task complete - All acceptance criteria met:
1. ✅ hoop-daemon builds without errors
2. ✅ Target directory `.beads/` confirmed to exist
3. ✅ Current clippy warnings documented (88 errors across 7 categories)
4. ✅ Ready to proceed with filtered clippy run
