# Redundant Pattern Priority List

**Generated:** 2026-08-13  
**Purpose:** Rank files by warning density to prioritize code cleanup efforts.

## Methodology

- **Warning Density** = (Total Warnings / Lines of Code) × 1000
- Higher density = more warnings per line of code = higher priority for cleanup
- Files with same density are ranked by total warning count

## Top 10 Files by Warning Density

| Rank | File | Lines | Warnings | Density (warnings/1K lines) | Primary Issues |
|------|------|-------|----------|------------------------------|----------------|
| 1 | `hoop-daemon/src/log_rotation.rs` | 293 | 3 | **10.24** | 3× disallowed_methods (File::create) |
| 2 | `hoop-daemon/src/api_blame.rs` | 382 | 2 | **5.24** | 2× manual_strip |
| 3 | `hoop-daemon/src/api_diff.rs` | 603 | 2 | **3.32** | 2× manual_strip |
| 4 | `hoop-daemon/src/uploads.rs` | 676 | 6 | **8.88** | 6× disallowed_methods (fs::write, File::create) |
| 5 | `hoop-daemon/src/capacity.rs` | 3333 | 8 | **2.40** | 5× dead_code, 2× unnecessary_unwrap, 1× explicit_counter_loop |
| 6 | `hoop-daemon/src/pdf_sanitize.rs` | 634 | 5 | **7.89** | 5× ptr_arg (&mut Vec vs &mut [_]) |
| 7 | `hoop-daemon/src/screen_capture.rs` | 709 | 5 | **7.05** | 5× disallowed_methods (fs::write, File::create) |
| 8 | `hoop-daemon/src/config_resolver.rs` | 2795 | 3 | **1.07** | 2× too_many_arguments, 1× if_same_then_else |
| 9 | `hoop-daemon/src/sessions.rs` | 3791 | 3 | **0.79** | 2× dead_code, 1× explicit_counter_loop |
| 10 | `hoop-daemon/src/fleet.rs` | 8504 | 3 | **0.35** | 3× too_many_arguments |

**Note:** Rankings 2 and 3 swapped by density - api_blame (5.24) > api_diff (3.32).

---

## Detailed Priority Analysis

### Priority 1: 🚨 Critical Density (≥5 warnings/1K lines)

**Files to address first:** `log_rotation.rs`, `api_blame.rs`, `api_diff.rs`

**Rationale:** These files have the highest concentration of issues. Quick wins that significantly improve code quality.

#### 1. `hoop-daemon/src/log_rotation.rs` (10.24/1K lines)
- **Issues:** 3× `disallowed_methods` - `std::fs::File::create` (lines 110, 117, 145)
- **Impact:** Security/compliance violation - direct file writes bypass atomic write safety
- **Fix effort:** LOW - Replace with `atomic_write::write_file()` or approved fs abstraction
- **Recommended action:** Immediate refactoring to use atomic write pattern

#### 2. `hoop-daemon/src/api_blame.rs` (5.24/1K lines)
- **Issues:** 2× `manual_strip` - prefix stripping (lines 228, 237)
- **Impact:** Code clarity - using `strip_prefix()` is more idiomatic and safer
- **Fix effort:** LOW - Simple string method replacement
- **Recommended action:** Quick refactor during next touch

#### 3. `hoop-daemon/src/api_diff.rs` (3.32/1K lines)
- **Issues:** 2× `manual_strip` - prefix stripping (lines 169, 178)
- **Impact:** Code clarity - same as api_blame
- **Fix effort:** LOW - Simple string method replacement
- **Recommended action:** Quick refactor during next touch

---

### Priority 2: ⚠️ High Density (3-5 warnings/1K lines)

**Files:** `uploads.rs`, `pdf_sanitize.rs`, `screen_capture.rs`

**Rationale:** High concentration of fixable issues. Address after critical density files.

#### 4. `hoop-daemon/src/uploads.rs` (8.88/1K lines)
- **Issues:** 6× `disallowed_methods` - `std::fs::write` (5×), `File::create` (1×)
- **Impact:** Security/compliance - direct writes bypass safety checks
- **Fix effort:** LOW-MEDIUM - Need to replace fs calls with atomic write pattern
- **Recommended action:** Batch refactor as part of FS safety compliance push

#### 5. `hoop-daemon/src/pdf_sanitize.rs` (7.89/1K lines)
- **Issues:** 5× `ptr_arg` - `&mut Vec` instead of `&mut [_]` (lines 184, 222, 259, 279, 311)
- **Impact:** Performance - unnecessary Vec allocation when slice suffices
- **Fix effort:** LOW - Simple type signature changes
- **Recommended action:** Quick refactor during next performance pass

#### 6. `hoop-daemon/src/screen_capture.rs` (7.05/1K lines)
- **Issues:** 5× `disallowed_methods` - `std::fs::write` (4×), `File::create` (1×)
- **Impact:** Security/compliance - direct writes bypass safety checks
- **Fix effort:** LOW-MEDIUM - Replace with atomic write pattern
- **Recommended action:** Batch refactor with uploads.rs as part of FS safety push

---

### Priority 3: 📊 Medium Density (1-3 warnings/1K lines)

**Files:** `capacity.rs`, `config_resolver.rs`, `sessions.rs`, `fleet.rs`

**Rationale:** Lower density but higher total warning count. May require significant refactoring.

#### 7. `hoop-daemon/src/capacity.rs` (2.40/1K lines)
- **Issues:** 5× `dead_code` (unused fields/structs), 2× `unnecessary_unwrap`, 1× `explicit_counter_loop`
- **Impact:** Code hygiene - unused code creates maintenance burden; unnecessary unwrap is a code smell
- **Fix effort:** MEDIUM - Dead code removal requires careful verification; unwrap fixes are trivial
- **Recommended action:** Incremental cleanup - remove dead code during maintenance work

#### 8. `hoop-daemon/src/config_resolver.rs` (1.07/1K lines)
- **Issues:** 2× `too_many_arguments` (9/7 args, lines 679, 1677), 1× `if_same_then_else`
- **Impact:** Code quality - functions with too many args are hard to maintain; duplicate if blocks are error-prone
- **Fix effort:** MEDIUM-HIGH - Requires refactoring to extract parameter structs or use builder pattern
- **Recommended action:** Defer to major refactoring pass or API redesign

#### 9. `hoop-daemon/src/sessions.rs` (0.79/1K lines)
- **Issues:** 2× `dead_code`, 1× `explicit_counter_loop`
- **Impact:** Code hygiene - unused code; explicit counter could be iterator
- **Fix effort:** MEDIUM - Dead code removal; counter may require loop restructuring
- **Recommended action:** Incremental cleanup during session-related work

#### 10. `hoop-daemon/src/fleet.rs` (0.35/1K lines)
- **Issues:** 3× `too_many_arguments` (12/7, 8/7, 8/7 args)
- **Impact:** Code quality - Complex function signatures are hard to maintain
- **Fix effort:** HIGH - Large file (8504 lines); requires parameter struct extraction
- **Recommended action:** Defer to major refactoring pass or API redesign

---

## Lower Priority Files (Density < 0.5 warnings/1K lines)

| File | Lines | Warnings | Density | Notes |
|------|-------|----------|---------|-------|
| `hoop-daemon/src/pattern_query_evaluator.rs` | 734 | 2 | 0.27 | 2× private_interfaces |
| `hoop-daemon/src/supervisor.rs` | 1383 | 2 | 0.14 | 1× too_many_arguments, 1× doc_overindented_list_items |
| `hoop-daemon/src/atomic_write.rs` | 675 | 2 | 0.30 | 2× disallowed_methods |
| `hoop-daemon/src/lib.rs` | 4152 | 2 | 0.48 | 1× dead_code, 1× unnecessary_sort_by |
| `hoop-daemon/src/api_screen_capture.rs` | 517 | 3 | 0.58 | 3× disallowed_methods |
| `hoop-cli/src/script.rs` | 264 | 2 | 0.76 | 2× dead_code |
| `hoop-daemon/src/stuck_detector.rs` | 1339 | 1 | 0.07 | 1× should_implement_trait |
| `hoop-daemon/src/config_watcher.rs` | 1207 | 1 | 0.08 | 1× large_enum_variant |
| `hoop-daemon/src/backup_pipeline.rs` | 946 | 1 | 0.11 | 1× disallowed_methods |
| `hoop-daemon/src/template_library.rs` | 627 | 1 | 0.16 | 1× disallowed_methods |
| `hoop-daemon/src/metrics.rs` | 1682 | 1 | 0.05 | 1× disallowed_methods |
| `hoop-daemon/src/projects.rs` | 1259 | 1 | 0.08 | 1× disallowed_methods |
| `hoop-daemon/src/parse_jsonl_safe.rs` | 455 | 1 | 0.22 | 1× disallowed_methods |
| `hoop-daemon/src/attachments.rs` | 1180 | 1 | 0.08 | 1× disallowed_methods |
| `hoop-daemon/src/attachment_sync.rs` | 790 | 1 | 0.13 | 1× disallowed_methods |
| `hoop-daemon/src/identity.rs` | 298 | 1 | 0.34 | 1× len_without_is_empty |
| `hoop-daemon/src/api_stitch_read.rs` | 588 | 1 | 0.17 | 1× unnecessary_sort_by |
| `hoop-daemon/src/api_unassigned.rs` | 464 | 1 | 0.22 | 1× unnecessary_sort_by |
| `hoop-daemon/src/api_conversations.rs` | 377 | 1 | 0.27 | 1× unnecessary_sort_by |
| `hoop-daemon/src/api_onboarding.rs` | 661 | 1 | 0.15 | 1× unnecessary_sort_by |
| `hoop-daemon/src/stitch_percentile_index.rs` | 1028 | 1 | 0.10 | 1× dead_code |
| `hoop-daemon/src/reflection_detector.rs` | 873 | 1 | 0.11 | 1× private_interfaces |
| `hoop-cli/src/config.rs` | 433 | 1 | 0.23 | 1× dead_code |
| `hoop-cli/src/projects.rs` | 1559 | 1 | 0.06 | 1× dead_code |
| `hoop-cli/src/init.rs` | 1045 | 1 | 0.10 | 1× non_snake_case |

---

## Recommended Action Plan

### Phase 1: Quick Wins (Density ≥5 warnings/1K lines)
**Estimated effort:** 2-3 hours  
**Files:** `log_rotation.rs`, `api_blame.rs`, `api_diff.rs`

- [ ] Refactor `log_rotation.rs` to use atomic write pattern
- [ ] Replace manual strip with `strip_prefix()` in api files
- [ ] Verify clippy passes after changes

### Phase 2: FS Safety Compliance (Density 3-8 warnings/1K lines)
**Estimated effort:** 4-6 hours  
**Files:** `uploads.rs`, `screen_capture.rs`, other `disallowed_methods` files

- [ ] Replace all `std::fs::write` and `File::create` with atomic write pattern
- [ ] Batch refactor all affected files as single compliance push
- [ ] Update FS operation guidelines to reflect new pattern

### Phase 3: Code Hygiene (Medium density, dead code focus)
**Estimated effort:** 6-8 hours  
**Files:** `capacity.rs`, `sessions.rs`, other dead code warnings

- [ ] Remove dead code (unused fields, structs, functions)
- [ ] Fix `unnecessary_unwrap` issues
- [ ] Refactor explicit counter loops to idiomatic iterators

### Phase 4: Structural Refactoring (Low density, high impact)
**Estimated effort:** 12-16 hours (defer to major refactoring)  
**Files:** `config_resolver.rs`, `fleet.rs`, `supervisor.rs`

- [ ] Extract parameter structs for functions with too many arguments
- [ ] Consider builder pattern for complex initialization
- [ ] Schedule as part of API redesign or major version bump

---

## Summary Statistics

- **Total files with warnings:** 33
- **Total warnings:** 75
- **Average density:** 1.24 warnings/1K lines (median: 0.22)
- **High-priority files (≥5 warnings/1K):** 3 files, 7 warnings total
- **Medium-priority files (1-5 warnings/1K):** 7 files, 28 warnings total
- **Low-priority files (<1 warnings/1K):** 23 files, 40 warnings total

**Quick win opportunity:** Fixing the top 3 critical density files eliminates 9% of warnings (7/75) in just 3% of the codebase, requiring minimal effort.

---

## Appendix: Raw Data

Source file: `docs/redundant-pattern-by-file.json`  
Line counts generated via: `wc -l hoop-daemon/src/*.rs hoop-cli/src/*.rs`