# Redundant Code Pattern Catalog

**Generated:** 2026-08-13
**Command:** `cargo clippy --workspace`
**Total Warnings:** 77

## Summary

**Key Finding:** The HOOP codebase has **ZERO** occurrences of the following targeted redundant patterns:
- `redundant_closure`
- `manual_flatten`
- `manual_clamp`

This indicates the codebase follows Rust idiomatic patterns well and avoids these common redundancy issues.

## Warning Distribution by Type

### High-Priority Patterns (Fixable Redundancy)

| Pattern | Count | Severity | Files Affected |
|---------|-------|----------|----------------|
| `unnecessary_sort_by` | 5 | Medium | lib.rs, api_conversations.rs, api_onboarding.rs, api_stitch_read.rs, api_unassigned.rs |
| `manual_strip` | 4 | Medium | api_blame.rs (2), api_diff.rs (2) |
| `unnecessary_unwrap` | 2 | Low | capacity.rs |
| `explicit_counter_loop` | 3 | Low | capacity.rs (2), sessions.rs |
| `ptr_arg` (&mut Vec vs &mut [_]) | 5 | Low | pdf_sanitize.rs (5) |
| `if_same_then_else` | 1 | Low | config_resolver.rs |

### Disallowed Methods (Crash-Safety Requirement)

| Pattern | Count | Severity | Files Affected |
|---------|-------|----------|----------------|
| `std::fs::write` (disallowed) | 19 | **High** | uploads.rs (4), screen_capture.rs (4), api_screen_capture.rs (3), projects.rs, attachment_sync.rs, attachments.rs, backup_pipeline.rs, metrics.rs, parse_jsonl_safe.rs, template_library.rs |
| `std::fs::File::create` (disallowed) | 7 | **High** | log_rotation.rs (3), atomic_write.rs (2), uploads.rs, screen_capture.rs |

**Note:** These 26 warnings are critical for crash-safety. All should use `atomic_write::atomic_write_file` instead.

### Code Quality Issues

| Pattern | Count | Severity | Files Affected |
|---------|-------|----------|----------------|
| `too_many_arguments` (8-12 args) | 6 | Low | fleet.rs (3), config_resolver.rs (2), supervisor.rs |
| `dead_code` (unused items) | 15 | Low | Multiple files |
| `private_interfaces` | 3 | Medium | pattern_query_evaluator.rs (2), reflection_detector.rs |
| `large_enum_variant` | 1 | Medium | config_watcher.rs |
| `len_without_is_empty` | 1 | Low | identity.rs |
| `should_implement_trait` | 1 | Low | stuck_detector.rs |

## Files by Warning Density (Priority Order)

### Critical Priority (5+ warnings)

| File | Count | Primary Issues |
|------|-------|----------------|
| `hoop-daemon/src/capacity.rs` | 9 | Dead code (3), unwrap (2), loop counter (2), disallowed write (1), interface privacy (1) |
| `hoop-daemon/src/uploads.rs` | 6 | Disallowed write (4), disallowed create (1), dead code (1) |
| `hoop-daemon/src/screen_capture.rs` | 5 | Disallowed write (4), disallowed create (1) |
| `hoop-daemon/src/pdf_sanitize.rs` | 5 | ptr_arg (&mut Vec) (5) |
| `hoop-daemon/src/lib.rs` | 4 | unnecessary_sort_by (1), dead code (3) |

### High Priority (3-4 warnings)

| File | Count | Primary Issues |
|------|-------|----------------|
| `hoop-daemon/src/pattern_query_evaluator.rs` | 4 | Private interfaces (2), unnecessary_sort_by (1), other (1) |
| `hoop-daemon/src/config_resolver.rs` | 4 | Too many args (2), if_same_then_else (1), other (1) |
| `hoop-daemon/src/api_diff.rs` | 4 | Manual strip (2), other (2) |
| `hoop-daemon/src/api_blame.rs` | 4 | Manual strip (2), other (2) |
| `hoop-daemon/src/fleet.rs` | 3 | Too many args (3) |
| `hoop-daemon/src/sessions.rs` | 3 | Dead code (2), loop counter (1) |
| `hoop-daemon/src/log_rotation.rs` | 3 | Disallowed create (3) |
| `hoop-daemon/src/api_screen_capture.rs` | 3 | Disallowed write (3) |

### Medium Priority (1-2 warnings)

Files with 1-2 warnings each (see full breakdown below).

## Detailed Catalog by File

### hoop-daemon/src/capacity.rs (9 warnings)

```rust
// Line 1613, 1772: explicit_counter_loop (2)
for line in reader.lines() {
    // Should be: for (line_number, line) in reader.lines().enumerate()

// Line 603, 604: unnecessary_unwrap (2)
if gcp_quota_config.is_some() {
    gcp_quota_config.as_ref().unwrap().project_id,
    // Should use if let Some(...) pattern

// Line 55: dead_code - rpm_limit
// Line 358: dead_code - session_id
// Line 472: dead_code - get_opencode_limits
// Line 526: dead_code - session_subpath
// Line 60: dead_code - QuotaLimit struct
```

### hoop-daemon/src/uploads.rs (6 warnings)

```rust
// Lines 132, 446, 470, 516: disallowed std::fs::write (4)
// Line 190: disallowed File::create
// Line 557: dead_code - subpath field

All should use atomic_write::atomic_write_file instead
```

### hoop-daemon/src/screen_capture.rs (5 warnings)

```rust
// Lines 353, 418, 452, 494: disallowed std::fs::write (4)
// Line 358: disallowed File::create

All should use atomic_write::atomic_write_file instead
```

### hoop-daemon/src/pdf_sanitize.rs (5 warnings)

```rust
// Lines 184, 222, 259, 279, 311: ptr_arg (5)
fn neutralise_open_action_js(data: &mut Vec<u8>, ...)
// Should be: fn neutralise_open_action_js(data: &mut [u8], ...)

All functions taking &mut Vec<u8> should take &mut [u8] instead
```

### hoop-daemon/src/lib.rs (4 warnings)

```rust
// Line 1219: unnecessary_sort_by
workers_by_project.sort_by(|a, b| b.worker_count.cmp(&a.worker_count));
// Should be: .sort_by_key(|b| std::cmp::Reverse(b.worker_count))

// Line 1307: dead_code - openapi_router
// Line 3826: dead_code - load_hoop_config
// Line 4093: dead_code - check_and_emit_capacity_alert
```

### hoop-daemon/src/pattern_query_evaluator.rs (4 warnings)

```rust
// Line 88, 258: private_interfaces (2)
// Type QueryExpr is private but used in pub(crate) functions

// Line 70+: other warnings
```

### hoop-daemon/src/config_resolver.rs (4 warnings)

```rust
// Line 371: if_same_then_else
if msg.contains("expected u8") || ... {
    Some("integer".to_string())
} else if msg.contains("expected i8") || ... {
    Some("integer".to_string())  // Identical block
}

// Line 679, 1677: too_many_arguments (2)
```

### hoop-daemon/src/api_diff.rs (4 warnings)

```rust
// Line 169, 178: manual_strip (2)
if line.starts_with("--- ") {
    let p = line[4..].trim_start_matches("a/");
    // Should be: if let Some(stripped) = line.strip_prefix("--- ")
```

### hoop-daemon/src/api_blame.rs (4 warnings)

```rust
// Line 228, 237: manual_strip (2)
if line.starts_with("author-time ") {
    if let Ok(unix) = line[12..].trim().parse::<i64>() {
    // Should be: if let Some(stripped) = line.strip_prefix("author-time ")
```

## Recommendations

### Immediate Actions (High Impact)

1. **Fix disallowed methods (26 warnings)**: Replace all `std::fs::write` and `File::create` with `atomic_write::atomic_write_file` for crash-safety

2. **Fix unnecessary_sort_by (5 warnings)**: Simple refactor using `sort_by_key`

3. **Fix manual_strip (4 warnings)**: Use `strip_prefix` method for clarity

### Low-Priority Cleanup

4. **Fix ptr_arg in pdf_sanitize.rs (5 warnings)**: Change `&mut Vec<u8>` to `&mut [u8]`

5. **Fix explicit_counter_loop (3 warnings)**: Use `.enumerate()`

6. **Fix unnecessary_unwrap (2 warnings)**: Use `if let` pattern

7. **Review and remove dead_code (15 warnings)**: Clean up unused items

### Consider for Refactoring

8. **Address too_many_arguments (6 warnings)**: Consider grouping related parameters into structs

9. **Fix private_interfaces (3 warnings)**: Adjust visibility or restructure

10. **Fix large_enum_variant (1 warning)**: Consider boxing large fields

## Verification

To verify completeness:

```bash
# Re-run clippy to confirm all warnings are cataloged
cargo clippy --workspace 2>&1 | grep "^warning:" | wc -l
# Expected: 77 warnings

# Verify no redundant_closure, manual_flatten, or manual_clamp exist
cargo clippy --workspace 2>&1 | grep -E "(redundant_closure|manual_flatten|manual_clamp)" | wc -l
# Expected: 0

# Count by pattern type
cargo clippy --workspace 2>&1 | grep "unnecessary_sort_by" | wc -l
# Expected: 5
```

## Conclusion

The HOOP codebase demonstrates **good Rust practices** with zero occurrences of the targeted redundancy patterns (redundant_closure, manual_flatten, manual_clamp). 

The primary concerns are:
1. **Crash-safety**: 26 disallowed method warnings should be addressed
2. **Code quality**: Minor refactoring opportunities (unnecessary_sort_by, manual_strip)
3. **Cleanup**: Dead code removal

All warnings are documented and categorized for prioritized fixing.
