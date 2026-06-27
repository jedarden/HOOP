# bf-37psy: Fix unused functions and traits

## Status
**ALREADY COMPLETED** - Work was done in prior commits.

## Summary
All unused functions and traits listed in the task description have already been removed:

### Prior Commits
- `de4f668` - Remove unused EmbedderExt trait
- `c71f9e7` - Remove unused is_email_detection_enabled function
- `8d25a9b` - Remove orphaned test_escape_html_entities test
- `04e0e4a` - Remove unused validation functions in config_resolver.rs

### Items Removed
1. **hoop-daemon/src/config_resolver.rs**:
   - `yaml_validate_str` (line 985)
   - `yaml_validate_u64_range` (line 1022)
   - `yaml_validate_f64_range` (line 1059)

2. **hoop-daemon/src/api_files.rs**:
   - `escape_html_entities` (line 533)

3. **hoop-daemon/src/secrets_scanner.rs**:
   - `is_email_detection_enabled` (line 133)

4. **hoop-daemon/src/embedding_service.rs**:
   - `EmbedderExt` trait (line 458)

## Verification
- `cargo clippy --workspace` runs cleanly with no unused_function or unused_trait warnings
- All targeted code has been successfully removed
