# Redundant Pattern Priority List

**Generated:** 2026-08-13  
**Source:** `docs/redundant-pattern-by-file.json`  
**Methodology:** Files ranked by warning density (warnings per 1,000 lines of code)

## Top 10 Files by Warning Density

This list identifies files with the highest concentration of clippy warnings, making them high-priority targets for cleanup and refactoring.

| Rank | File | Warnings | Lines | Density (per 1K LOC) | Primary Warning Types |
|------|------|----------|-------|---------------------|----------------------|
| 1 | `hoop-daemon/src/log_rotation.rs` | 3 | 293 | 10.24 | disallowed_methods (×3) |
| 2 | `hoop-daemon/src/uploads.rs` | 6 | 676 | 8.87 | disallowed_methods (×6) |
| 3 | `hoop-daemon/src/pdf_sanitize.rs` | 5 | 634 | 7.89 | ptr_arg (×5) |
| 4 | `hoop-daemon/src/capacity.rs` | 9 | 3333 | 2.70 | dead_code (×5), unnecessary_unwrap (×2), explicit_counter_loop (×2) |
| 5 | `hoop-cli/src/script.rs` | 2 | 264 | 7.58 | dead_code (×2) |
| 6 | `hoop-daemon/src/screen_capture.rs` | 5 | 709 | 7.05 | disallowed_methods (×5) |
| 7 | `hoop-daemon/src/api_screen_capture.rs` | 3 | 517 | 5.80 | disallowed_methods (×3) |
| 8 | `hoop-daemon/src/api_blame.rs` | 2 | 382 | 5.24 | manual_strip (×2) |
| 9 | `hoop-daemon/src/identity.rs` | 1 | 298 | 3.36 | len_without_is_empty |
| 10 | `hoop-daemon/src/sessions.rs` | 3 | 3791 | 0.79 | dead_code (×2), explicit_counter_loop |

## Full Ranked List (All Files)

| Rank | File | Warnings | Lines | Density (per 1K LOC) | Primary Warning Types |
|------|------|----------|-------|---------------------|----------------------|
| 1 | `hoop-daemon/src/log_rotation.rs` | 3 | 293 | **10.24** | disallowed_methods (×3) |
| 2 | `hoop-daemon/src/uploads.rs` | 6 | 676 | **8.87** | disallowed_methods (×6) |
| 3 | `hoop-daemon/src/pdf_sanitize.rs` | 5 | 634 | **7.89** | ptr_arg (×5) |
| 4 | `hoop-cli/src/script.rs` | 2 | 264 | **7.58** | dead_code (×2) |
| 5 | `hoop-daemon/src/screen_capture.rs` | 5 | 709 | **7.05** | disallowed_methods (×5) |
| 6 | `hoop-daemon/src/api_screen_capture.rs` | 3 | 517 | **5.80** | disallowed_methods (×3) |
| 7 | `hoop-daemon/src/api_blame.rs` | 2 | 382 | **5.24** | manual_strip (×2) |
| 8 | `hoop-daemon/src/api_diff.rs` | 2 | 603 | **3.32** | manual_strip (×2) |
| 9 | `hoop-daemon/src/identity.rs` | 1 | 298 | **3.36** | len_without_is_empty |
| 10 | `hoop-daemon/src/capacity.rs` | 9 | 3333 | **2.70** | dead_code (×5), unnecessary_unwrap (×2), explicit_counter_loop (×2) |
| 11 | `hoop-daemon/src/atomic_write.rs` | 2 | 675 | **2.96** | disallowed_methods (×2) |
| 12 | `hoop-daemon/src/pattern_query_evaluator.rs` | 2 | 734 | **2.73** | private_interfaces (×2) |
| 13 | `hoop-cli/src/config.rs` | 1 | 433 | **2.31** | dead_code |
| 14 | `hoop-daemon/src/attachment_sync.rs` | 1 | 790 | **1.27** | disallowed_methods |
| 15 | `hoop-daemon/src/api_conversations.rs` | 1 | 377 | **2.65** | unnecessary_sort_by |
| 16 | `hoop-daemon/src/api_onboarding.rs` | 1 | 661 | **1.51** | unnecessary_sort_by |
| 17 | `hoop-daemon/src/api_stitch_read.rs` | 1 | 588 | **1.70** | unnecessary_sort_by |
| 18 | `hoop-daemon/src/api_unassigned.rs` | 1 | 464 | **2.16** | unnecessary_sort_by |
| 19 | `hoop-daemon/src/reflection_detector.rs` | 1 | 873 | **1.15** | private_interfaces |
| 20 | `hoop-daemon/src/parse_jsonl_safe.rs` | 1 | 455 | **2.20** | disallowed_methods |
| 21 | `hoop-daemon/src/supervisor.rs` | 2 | 1383 | **1.45** | too_many_arguments, doc_overindented_list_items |
| 22 | `hoop-daemon/src/config_resolver.rs` | 3 | 2795 | **1.07** | if_same_then_else, too_many_arguments (×2) |
| 23 | `hoop-daemon/src/stuck_detector.rs` | 1 | 1339 | **0.75** | should_implement_trait |
| 24 | `hoop-daemon/src/template_library.rs` | 1 | 627 | **1.59** | disallowed_methods |
| 25 | `hoop-daemon/src/stitch_percentile_index.rs` | 1 | 1028 | **0.97** | dead_code |
| 26 | `hoop-daemon/src/backup_pipeline.rs` | 1 | 946 | **1.06** | disallowed_methods |
| 27 | `hoop-daemon/src/attachments.rs` | 1 | 1180 | **0.85** | disallowed_methods |
| 28 | `hoop-daemon/src/metrics.rs` | 1 | 1682 | **0.59** | disallowed_methods |
| 29 | `hoop-daemon/src/projects.rs` | 1 | 1259 | **0.79** | disallowed_methods |
| 30 | `hoop-daemon/src/lib.rs` | 4 | 4152 | **0.96** | dead_code (×3), unnecessary_sort_by |
| 31 | `hoop-daemon/src/sessions.rs` | 3 | 3791 | **0.79** | dead_code (×2), explicit_counter_loop |
| 32 | `hoop-daemon/src/config_watcher.rs` | 1 | 1207 | **0.83** | large_enum_variant |
| 33 | `hoop-cli/src/init.rs` | 1 | 1045 | **0.96** | non_snake_case |
| 34 | `hoop-cli/src/projects.rs` | 1 | 1559 | **0.64** | dead_code |
| 35 | `hoop-daemon/src/fleet.rs` | 3 | 8504 | **0.35** | too_many_arguments (×3) |

## Prioritization Rationale

### High Priority (Density > 5.0)
These files have the highest concentration of warnings and should be addressed first:

1. **`log_rotation.rs`** (10.24/1K LOC): All 3 warnings are disallowed `std::fs::File::create` calls that should use atomic write patterns
2. **`uploads.rs`** (8.87/1K LOC): 6 disallowed `std::fs::write` calls — file I/O should be refactored to use atomic writes
3. **`pdf_sanitize.rs`** (7.89/1K LOC): 5 `ptr_arg` warnings indicate repeated slice/vec anti-patterns that should be refactored for better idiomatic Rust
4. **`script.rs`** (7.58/1K LOC): 2 dead code warnings in a small file suggests incomplete implementation or vestigial code

### Medium Priority (Density 1.0 - 5.0)
These files have moderate warning density but may contain architectural issues:

- **`capacity.rs`**: 9 warnings including 5 dead code items — suggests incomplete refactoring or deprecated code paths
- **`screen_capture.rs`** & **`api_screen_capture.rs`**: Multiple disallowed method warnings — file I/O patterns need atomic write refactoring
- **`api_blame.rs`** & **`api_diff.rs`**: Manual `strip_prefix` calls should use the standard `strip_prefix()` method
- **`identity.rs`**: Missing `is_empty()` method is a public API incompleteness

### Low Priority (Density < 1.0)
These files have low warning density but may contain isolated issues worth addressing:

- **`fleet.rs`**: Despite low density (0.35/1K LOC), it has 3 functions with too many arguments — potential design smell requiring structural refactoring
- **`config_resolver.rs`**: `if_same_then_else` warning indicates duplicated logic that should be extracted
- **`stuck_detector.rs`**: `should_implement_trait` suggests implementing `FromStr` trait for better ergonomics

## Recommended Action Order

1. **Quick Wins (Disallowed Methods)**: Fix all `disallowed_methods` warnings first — these are straightforward mechanical replacements with clear migration paths (use atomic write utilities)
2. **Dead Code Removal**: Address `dead_code` warnings in small files first (`script.rs`, `capacity.rs`) — these suggest incomplete implementation
3. **API Completeness**: Fix missing trait implementations (`identity.rs`, `stuck_detector.rs`) — these affect public API ergonomics
4. **Structural Issues**: Address `too_many_arguments` warnings in `fleet.rs`, `config_resolver.rs`, and `supervisor.rs` — these require deeper design consideration
5. **Style Improvements**: Fix `ptr_arg`, `manual_strip`, and `unnecessary_sort_by` warnings — these are code quality improvements

## Summary Statistics

- **Total files with warnings**: 35
- **Total warnings**: 75
- **Average warning density**: 2.14 per 1,000 LOC
- **Highest density**: 10.24 per 1,000 LOC (`log_rotation.rs`)
- **Most warnings in single file**: 9 (`capacity.rs`)
- **Most common warning type**: disallowed_methods (28 occurrences)

## Notes

- Warning density is calculated as: `(warnings / lines_of_code) × 1000`
- Line counts include all lines (code, comments, blanks)
- Files with fewer than 200 lines are excluded from density ranking to avoid skewing
- This analysis should be updated after each cleanup pass to track progress
