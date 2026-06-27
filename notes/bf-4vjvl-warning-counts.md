# HOOP Clippy Warning Count Summary

## Total Warnings: ~200+ (with compilation errors mixed in)

## Warning Counts by Category

### HIGH PRIORITY (31 warnings)
- disallowed_methods (std::fs::write, std::fs::File::create): 30
- await_holding_lock: 1

### MEDIUM PRIORITY (34 warnings)
- dead_code (unused functions/fields/constants/structs): 20
- type_complexity: 5
- large_enum_variant: 1
- private_interfaces: 1
- too_many_arguments: 4
- new_without_default: 1
- should_implement_trait: 2
- len_without_is_empty: 1

### LOW PRIORITY (135+ warnings)
- derivable_impls: 5
- needless_borrow: 10
- redundant_closure: 8
- manual_clamp: 3
- unnecessary_sort_by: 6
- manual_flatten: 6
- collapsible_if: 3
- manual_range_contains: 1
- manual_strip: 4
- single_match: 1
- useless_format: 3
- useless_conversion: 10
- explicit_counter_loop: 3
- ptr_arg: 5
- clone_on_copy: 3
- map_flatten: 2
- unnecessary_map_or: 5
- unnecessary_lazy_evaluations: 2
- get_first: 1
- needless_return: 1
- bind_instead_of_map: 1
- double_ended_iterator_last: 1
- for_kv_map: 1
- match_result_ok: 1
- cast_abs_to_unsigned: 1
- manual_pattern_char_comparison: 3
- doc_overindented_list_items: 1
- explicit_auto_deref: 2
- mismatched_lifetime_syntaxes: 1
- unused_variables: 15
- unused_imports: 15
- unnecessary_cast: 15

## Files with Most Warnings (Top 10)

1. hoop-daemon/src/lib.rs - 20+ warnings
2. hoop-daemon/src/config_resolver.rs - 10+ warnings
3. hoop-daemon/src/capacity.rs - 10+ warnings
4. hoop-daemon/src/backup_pipeline.rs - 8 warnings
5. hoop-daemon/src/api_conversations.rs - 6 warnings
6. hoop-daemon/src/api_scripts.rs - 6 warnings
7. hoop-daemon/src/api_skills.rs - 6 warnings
8. hoop-daemon/src/uploads.rs - 5 warnings
9. hoop-daemon/src/projects.rs - 5 warnings
10. hoop-daemon/src/screen_capture.rs - 5 warnings

## High Severity Files (Crash Safety Issues)

Files with disallowed_methods warnings that MUST be fixed:
- agent_session.rs
- attachments.rs
- uploads.rs
- screen_capture.rs
- atomic_write.rs
- log_rotation.rs
- api_screen_capture.rs
- template_library.rs
- projects.rs
- backup_pipeline.rs
- attachment_sync.rs
- metrics.rs
- parse_jsonl_safe.rs
- dictated_notes.rs
- api_unassigned.rs

Total files with crash-safety warnings: 15

## Auto-fixable Warnings

Clippy reports that 97 suggestions can be auto-applied via:
```bash
cargo clippy --fix --lib -p hoop-daemon
```

However, the high-priority disallowed_methods require manual review.
