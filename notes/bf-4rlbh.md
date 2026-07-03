# Unused Functions Analysis: config_resolver.rs

Bead ID: bf-4rlbh
Date: 2026-07-03

## Summary

Analyzed `hoop-daemon/src/config_resolver.rs` (2404 lines) for unused functions. Found 7 functions that are either completely unused or only used by other unused functions.

## Unused Functions

### 1. `SecretPattern::flatten_patterns` (Line 62)
**Status:** Completely unused
- **Purpose:** Flatten a list of `SecretPattern` objects into a list of regex strings
- **Defined as:** `pub fn flatten_patterns(patterns: &[SecretPattern]) -> Vec<String>`
- **Search result:** No references found in codebase
- **Recommended action:** Remove or add to TODO list if future use is intended

### 2. `yaml_get_redaction_policy` (Line 872)
**Status:** Marked `#[allow(dead_code)]`, completely unused
- **Purpose:** Extract redaction policy configuration from YAML
- **Returns:** `Option<crate::redaction_policy::GlobalRedactionPolicy>`
- **Search result:** No references found in codebase
- **Note:** Related to `redaction_policy` module which may be unimplemented

### 3. `resolve_opt_strict` (Line 679)
**Status:** Marked `#[allow(dead_code)]`, completely unused
- **Purpose:** Strict resolve with type validation for hot-reload
- **Returns:** `Result<Resolved<T>, ConfigError>`
- **Search result:** No references found in codebase
- **Note:** Intended for config hot-reload validation (§17.5), but the simpler `resolve_from_raw` is used instead

### 4. `yaml_validate_bool` (Line 929)
**Status:** Marked `#[allow(dead_code)]`, unused
- **Purpose:** Strictly validate a boolean field
- **Search result:** Only called by other unused validation functions
- **Dependency:** Uses `yaml_navigate`

### 5. `yaml_validate_u64` (Line 949)
**Status:** Marked `#[allow(dead_code)]`, unused
- **Purpose:** Strictly validate an integer field
- **Search result:** Only called by other unused validation functions
- **Dependency:** Uses `yaml_navigate`

### 6. `yaml_validate_f64` (Line 969)
**Status:** Marked `#[allow(dead_code)]`, unused
- **Purpose:** Strictly validate a float field
- **Search result:** Only called by other unused validation functions
- **Dependency:** Uses `yaml_navigate`

### 7. `yaml_navigate` (Line 915)
**Status:** Marked `#[allow(dead_code)]`, only used by unused functions
- **Purpose:** Navigate to a nested YAML value by dotted path
- **Search result:** Only called by `yaml_validate_bool`, `yaml_validate_u64`, `yaml_validate_f64` (all unused)
- **Note:** Superseded by the simpler `yaml_get_*` helpers

## Functions That ARE Used (for reference)

The following functions were checked and ARE used elsewhere:
- `to_named_patterns` - Used in `lib.rs` and `redaction.rs`
- `validate_agent_adapter` - Used in `resolve_from_raw`
- `validate_ui_theme` - Used in `resolve_from_raw`
- `validate_ui_sort` - Used in `resolve_from_raw`
- `validate_embedding_adapter` - Used in `resolve_from_raw`
- `parse_serde_yaml_details` - Used internally by `ConfigError::from_yaml`
- `validate_schema_version` - Used in `resolve_from_raw`

## Cleanup Recommendations

1. **Safe to remove immediately:**
   - `SecretPattern::flatten_patterns` (public but unused)
   - `yaml_get_redaction_policy` (already marked dead_code)
   - `resolve_opt_strict` (already marked dead_code)
   - `yaml_validate_bool` (already marked dead_code)
   - `yaml_validate_u64` (already marked dead_code)
   - `yaml_validate_f64` (already marked dead_code)
   - `yaml_navigate` (already marked dead_code)

2. **Code smell:** The `#[allow(dead_code)]` attributes suggest this was planned code for config hot-reload validation that was never fully implemented. The actual `resolve_from_raw` function uses a simpler validation approach via `resolve_validated_str` helper instead.

3. **Total savings:** Removing these 7 functions would save approximately 120 lines of code.
