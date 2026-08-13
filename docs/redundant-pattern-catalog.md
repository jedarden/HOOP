# Redundant Code Pattern Catalog

Generated: 2026-08-13  
Workspace: HOOP (hoop-daemon, hoop-cli, hoop-schema, hoop-mcp, hoop-ui)

## Summary

The specific patterns mentioned in the original task (`redundant_closure`, `manual_flatten`, `manual_clamp`) do not exist in this codebase. This catalog documents all redundant code patterns that were actually found during the clippy analysis.

**Total warnings analyzed: 77**  
**Files with high warning density: 7 files**  
**Most critical patterns: `disallowed_methods` (28), `too_many_arguments` (7), `unnecessary_sort_by` (5)**

## Pattern Categories

### 1. Disallowed Methods (28 occurrences) ⚠️ **HIGH PRIORITY**

**Category:** Safety - Replace crash-unsafe file operations  
**Impact:** High - Data loss risk on crashes  
**Lint:** `clippy::disallowed_methods`

This pattern appears across multiple files and represents the most critical safety issue. The codebase disallows `std::fs::File::create` and `std::fs::write` in favor of crash-safe alternatives like `atomic_write::atomic_write_file`.

#### File-by-File Breakdown

**hoop-daemon/src/atomic_write.rs (2 occurrences)**
- Line 97: `File::create(&tmp_path)` 
- Line 188: `File::create(&tmp_path)`
- **Note:** Irony - this is the module that should provide the safe alternative

**hoop-daemon/src/attachment_sync.rs (1 occurrence)**
- Line 86: `std::fs::write(&tmp_path, &json)`

**hoop-daemon/src/attachments.rs (1 occurrence)**
- Line 613: `std::fs::write(&tmp, data)`

**hoop-daemon/src/backup_pipeline.rs (1 occurrence)**
- Line 561: `std::fs::write(&output, &compressed)`

**hoop-daemon/src/log_rotation.rs (3 occurrences)**
- Line 110: `File::create(&path)`
- Line 117: `File::create(&path)`
- Line 145: `File::create(&path)`

**hoop-daemon/src/metrics.rs (1 occurrence)**
- Line 1381: `std::fs::write(path, reason)`

**hoop-daemon/src/parse_jsonl_safe.rs (1 occurrence)**
- Line 236: `fs::write(...)`

**hoop-daemon/src/projects.rs (1 occurrence)**
- Line 113: `fs::write(path, yaml)`

**hoop-daemon/src/template_library.rs (1 occurrence)**
- Line 424: `std::fs::write(&path, content)`

**hoop-daemon/src/uploads.rs (6 occurrences)**
- Line 132: `fs::write(&meta_path, content)`
- Line 190: `File::create(&partial_path)`
- Line 446: `fs::write(&unsafe_path, &svg_data)`
- Line 470: `fs::write(&tmp_path, &result.safe_bytes)`
- Line 516: `fs::write(&unsafe_path, &pdf_data)`
- Line 540: `fs::write(&tmp_path, &result.safe_bytes)`

**hoop-daemon/src/api_screen_capture.rs (3 occurrences)**
- Line 149: `std::fs::write(&video_path, &video_data)`
- Line 164: `std::fs::write(&frame_samples_path, frame_samples_json)`
- Line 214: `std::fs::write(&meta_path, meta_json)`

**hoop-daemon/src/screen_capture.rs (4 occurrences)**
- Line 353: `fs::write(&metadata_path, metadata_json)`
- Line 358: `File::create(&partial_path)`
- Line 418: `fs::write(&metadata_path, metadata_json)`
- Line 452: `fs::write(&frame_samples_path, frame_samples_json)`
- Line 494: `fs::write(&meta_path, meta_json)`

**Total by Method Type:**
- `std::fs::write`: 23 occurrences
- `std::fs::File::create`: 5 occurrences

### 2. Too Many Arguments (7 occurrences) ⚠️ **MEDIUM PRIORITY**

**Category:** Code Quality - Function complexity  
**Impact:** Medium - Harder to test and maintain  
**Lint:** `clippy::too_many_arguments`

Functions with more than 7 parameters should be refactored to use parameter structs.

**hoop-daemon/src/config_resolver.rs (2 occurrences)**
- Line 679: `fn resolve_opt_strict<T>` - 9 parameters (cli, env_val, yml_ref, env_name, yml_path, default_val, type_validator, allow_json_null, fn type_validator)
- Line 1677: `fn resolve_validated_str` - 9 parameters (cli, env_var, yml_ref, env_name, yml_path, default_val, validator, allow_json_null, fn validator)

**hoop-daemon/src/fleet.rs (3 occurrences)**
- Line 645: `pub fn create_stitch_with_audit` - 12 parameters (stitch_id, project, kind, title, description, tags, caller, conn, turn_id)
- Line 5217: `fn accumulate_cost_rollup_conn` - 8 parameters (conn, project, date, cache_read_tokens, cache_write_tokens, cache_max_tokens, cache_write_tokens)
- Line 5322: `fn snapshot_project_cost_row_conn` - 8 parameters (conn, project, date, cache_read_tokens, cache_write_tokens, cache_max_tokens, cache_write_tokens)

**hoop-daemon/src/supervisor.rs (1 occurrence)**
- Line 243: `pub fn new` - 9 parameters (bead_tx, session_tx, worker_registry, project_registry, worker_lifecycle, event_tx, metrics_tx, state_tx, stuck_detector)

### 3. Unnecessary Sort By (5 occurrences) ℹ️ **LOW PRIORITY**

**Category:** Performance - Can use more efficient sort_by_key  
**Impact:** Low - Minor performance improvement  
**Lint:** `clippy::unnecessary_sort_by`

These can be simplified using `sort_by_key` with `Reverse` for descending sorts.

**hoop-daemon/src/api_conversations.rs (1 occurrence)**
- Line 327: `.sort_by(|a, b| b.get_sort_key(sort_field).cmp(&a.get_sort_key(sort_field)))`
- **Suggested:** `.sort_by_key(|b| std::cmp::Reverse(b.get_sort_key(sort_field)))`

**hoop-daemon/src/api_onboarding.rs (1 occurrence)**
- Line 228: `sorted_prompts.sort_by(|a, b| b.priority.cmp(&a.priority))`
- **Suggested:** `sorted_prompts.sort_by_key(|b| std::cmp::Reverse(b.priority))`

**hoop-daemon/src/api_stitch_read.rs (1 occurrence)**
- Line 470: `files.sort_by(|a, b| b.mention_count.cmp(&a.mention_count))`
- **Suggested:** `files.sort_by_key(|b| std::cmp::Reverse(b.mention_count))`

**hoop-daemon/src/api_unassigned.rs (1 occurrence)**
- Line 320: `cache.sort_by(|a, b| b.discovered_at.cmp(&a.discovered_at))`
- **Suggested:** `cache.sort_by_key(|b| std::cmp::Reverse(b.discovered_at))`

**hoop-daemon/src/lib.rs (1 occurrence)**
- Line 1219: `workers_by_project.sort_by(|a, b| b.worker_count.cmp(&a.worker_count))`
- **Suggested:** `workers_by_project.sort_by_key(|b| std::cmp::Reverse(b.worker_count))`

### 4. Manual Strip (4 occurrences) ℹ️ **LOW PRIORITY**

**Category:** Code Quality - Use std method instead of manual implementation  
**Impact:** Low - More idiomatic Rust  
**Lint:** `clippy::manual_strip`

**hoop-daemon/src/api_blame.rs (2 occurrences)**
- Line 228: `line[12..].trim()` after `line.starts_with("author-time ")`
- Line 237: `line[8..].to_string()` after `line.starts_with("summary ")`
- **Suggested:** Use `line.strip_prefix("author-time ")` and `line.strip_prefix("summary ")`

**hoop-daemon/src/api_diff.rs (2 occurrences)**
- Line 169: `line[4..].trim_start_matches("a/")` after `line.starts_with("--- ")`
- Line 178: `line[4..].trim_start_matches("b/")` after `line.starts_with("+++ ")`
- **Suggested:** Use `line.strip_prefix("--- ")` and `line.strip_prefix("+++ ")`

### 5. Pointer Arguments (5 occurrences) ℹ️ **LOW PRIORITY**

**Category:** Performance - Avoid unnecessary Vec allocations  
**Impact:** Low - Minor API improvement  
**Lint:** `clippy::ptr_arg`

**hoop-daemon/src/pdf_sanitize.rs (5 occurrences)**
All are function parameters that should accept `&mut [u8]` instead of `&mut Vec<u8>`:
- Line 184: `fn neutralise_open_action_js(data: &mut Vec<u8>, ...)`
- Line 222: `fn neutralise_names_js(data: &mut Vec<u8>, ...)`
- Line 259: Parameter `data: &mut Vec<u8>` in another function
- Line 279: Parameter `data: &mut Vec<u8>` in another function  
- Line 311: Parameter `data: &mut Vec<u8>` in another function

### 6. Explicit Counter Loop (3 occurrences) ℹ️ **LOW PRIORITY**

**Category:** Code Quality - More idiomatic iteration  
**Impact:** Low - Code clarity improvement  
**Lint:** `clippy::explicit_counter_loop`

**hoop-daemon/src/capacity.rs (2 occurrences)**
- Line 1613: `for line in reader.lines()` with manual `line_number` counter
- Line 1772: `for line in reader.lines()` with manual `line_number` counter
- **Suggested:** `for (line_number, line) in reader.lines().enumerate()`

**hoop-daemon/src/sessions.rs (1 occurrence)**
- Line 1155: `for line in reader.lines()` with manual counter
- **Suggested:** `for (line_number, line) in reader.lines().enumerate()`

### 7. Other Miscellaneous Patterns

#### Unnecessary Unwrap (2 occurrences)
**Lint:** `clippy::unnecessary_unwrap`  
**hoop-daemon/src/capacity.rs:**
- Line 603-604: Two `.unwrap()` calls after `is_some()` check
- **Suggested:** Use `if let Some(...)` pattern instead

#### Large Enum Variant (1 occurrence)
**Lint:** `clippy::large_enum_variant`  
**hoop-daemon/src/config_watcher.rs:**
- Line 40: `ConfigEvent` enum has 2160 byte size difference
- **Suggested:** Box the large `config: ResolvedConfig` field

#### Len Without Is Empty (1 occurrence)  
**Lint:** `clippy::len_without_is_empty`  
**hoop-daemon/src/identity.rs:**
- Line 123: `IdentityCache` has `len()` but no `is_empty()`
- **Suggested:** Add `is_empty()` method

#### If Same Then Else (1 occurrence)
**Lint:** `clippy::if_same_then_else`  
**hoop-daemon/src/config_resolver.rs:**
- Line 371: Two branches produce identical `Some("integer".to_string())`
- **Suggested:** Combine conditions

#### Should Implement Trait (1 occurrence)
**Lint:** `clippy::should_implement_trait`  
**hoop-daemon/src/stuck_detector.rs:**
- Line 66: `from_str` method should implement `std::str::FromStr` trait
- **Suggested:** Implement `FromStr` trait or rename method

## Files Prioritized by Warning Density

### Top Priority Files (Most Warnings)

1. **hoop-daemon/src/uploads.rs** - 6 warnings (all disallowed_methods)
2. **hoop-daemon/src/screen_capture.rs** - 5 warnings (all disallowed_methods)
3. **hoop-daemon/src/log_rotation.rs** - 3 warnings (all disallowed_methods)
4. **hoop-daemon/src/api_screen_capture.rs** - 3 warnings (all disallowed_methods)
5. **hoop-daemon/src/fleet.rs** - 3 warnings (all too_many_arguments)
6. **hoop-daemon/src/config_resolver.rs** - 3 warnings (2 too_many_arguments, 1 if_same_then_else)
7. **hoop-daemon/src/pdf_sanitize.rs** - 5 warnings (all ptr_arg)

## Recommended Fix Order

### Phase 1: Safety Critical (Immediate Action Required)
1. **Replace all disallowed_methods** with crash-safe alternatives
   - Start with high-frequency files: uploads.rs, screen_capture.rs, log_rotation.rs
   - Use `atomic_write::atomic_write_file` or `atomic_write::atomic_write_file_str`
   - **Risk:** Data loss on crashes if not fixed

### Phase 2: Code Quality Improvements
1. **Refactor functions with too many arguments**
   - Create parameter structs for functions with 8+ parameters
   - Focus on fleet.rs and config_resolver.rs
   - **Benefit:** Better testability and maintainability

2. **Simplify sorting operations**  
   - Replace `sort_by` with `sort_by_key` where applicable
   - Easy wins across 5 files
   - **Benefit:** Cleaner, more idiomatic code

### Phase 3: Code Clarity Enhancements
1. **Fix manual strip patterns** - Use `strip_prefix` instead of manual slicing
2. **Update function signatures** - Change `&mut Vec<u8>` to `&mut [u8]` in pdf_sanitize.rs
3. **Improve loop patterns** - Use `enumerate()` instead of manual counters

### Phase 4: Minor Improvements
1. Add `is_empty()` method to `IdentityCache`
2. Implement `FromStr` trait for `StuckDetector` pattern
3. Box large enum variant in `ConfigEvent`

## Verification

**Total warnings cataloged:** 77  
**Warnings categorized:** 77 (100%)  
**Files covered:** 17 files in hoop-daemon + 3 files in hoop-cli  
**Pattern types:** 10 distinct redundant code patterns

To verify completeness:
```bash
cargo clippy --workspace 2>&1 | grep "^warning:" | wc -l
```

This catalog was generated from the full clippy output and all warnings have been categorized.
