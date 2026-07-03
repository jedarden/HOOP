# bf-23yst: Fix unused functions in config_resolver.rs

## Status: Already Completed (Duplicate)

This bead is a duplicate of the already-closed bead `bf-35qvj`.

## What Was Done

The work was completed in commit `04e0e4a` (2025-06-27):
- Removed the three unused validation functions:
  - `yaml_validate_str` (originally line 985)
  - `yaml_validate_u64_range` (originally line 1022)
  - `yaml_validate_f64_range` (originally line 1059)
- Added `#[allow(dead_code)]` to unused-but-intended functions:
  - `resolve_opt_strict` (for hot-reload type validation)
  - `yaml_get_redaction_policy` (for future redaction config)
  - `yaml_navigate` (helper for strict validation)
  - `yaml_validate_bool` (strict type validator)
  - `yaml_validate_u64` (strict type validator)
  - `yaml_validate_f64` (strict type validator)

## Verification

The current file (`hoop-daemon/src/config_resolver.rs`) shows:
- No references to the three removed functions
- All remaining unused functions properly marked with `#[allow(dead_code)]`
- The `#[allow(dead_code)]` attributes are at lines: 678, 871, 914, 928, 948, 968

## Acceptance Criteria

Already satisfied: `cargo clippy` shows no `unused_function` warnings for `hoop-daemon/src/config_resolver.rs` after the changes in commit `04e0e4a`.
