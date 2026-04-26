# Hot-Reload Validator for config.yml and projects.yaml

## Overview

The HOOP daemon implements a comprehensive hot-reload validator for both `config.yml` and `projects.yaml` that validates configuration changes before applying them, rejecting invalid configs with detailed error messages while keeping the previous valid config running.

## Implementation Status

### ✅ Already Implemented

#### config.yml Hot-Reload (`hoop-daemon/src/config_watcher.rs`)
- **File watching**: Uses `notify` crate with 2-second debouncing
- **Validate-before-apply pipeline**:
  1. Read & parse YAML → structured
  2. Schema-validate (type checking via `resolve_from_raw`)
  3. Apply (store new config or rollback)
- **Rollback on error**: Previous valid config kept in memory
- **Structured error reporting**: `ConfigError` with message, line, col, field, expected, got
- **Metrics**: `hoop_config_reload_success_total`, `hoop_config_reload_rejected_total`
- **Audit trail**: Logs to fleet.db on success/failure

#### projects.yaml Hot-Reload (`hoop-daemon/src/projects.rs`)
- **File watching**: Uses `notify` crate with 5-second debouncing
- **Validate-before-apply pipeline**:
  1. Schema-validate (parse YAML into typed structure)
  2. Semantic-validate (paths exist, .beads directories, duplicate detection)
  3. Apply (store new config or rollback)
- **Rollback on error**: Previous valid config kept in memory
- **Structured error reporting**: Same `ConfigError` format as config.yml
- **Metrics**: Same metrics as config.yml
- **Audit trail**: Logs to fleet.db on success/failure

#### Validation Coverage

**Type Validation** (config_resolver.rs):
- `yaml_validate_bool()` - Boolean fields
- `yaml_validate_u64()` - Integer fields
- `yaml_validate_f64()` - Float fields
- `yaml_validate_str()` - String fields

**Semantic Validation**:
- `validate_agent_adapter()` - Valid adapters: claude, codex, opencode, gemini, aider
- `validate_ui_theme()` - Valid themes: auto, light, dark, solarized-light, solarized-dark
- `validate_ui_sort()` - Valid sorts: name, last_activity, cost_today, worker_count

**Unknown Field Detection**:
- Checks against whitelist of valid top-level keys
- Rejects config with unknown fields

**Secret Pattern Validation**:
- Validates regex patterns compile correctly
- Validates severity levels (high, medium, low)

**Projects Semantic Validation**:
- Workspace path existence checks
- `.beads` directory validation
- Duplicate canonical path detection
- Role validation

#### UI Banner System (`hoop-daemon/src/ws.rs`, `hoop-daemon/src/lib.rs`)
- **WebSocket broadcasts**: Config errors broadcast to all connected clients
- **Initial status**: New clients receive current config status on connection
- **Status persistence**: `config_status_state` stores current status for new connections
- **Structured error data**: `ConfigStatusData` with valid flag and `ConfigErrorData` details

### 🔧 Fixes Applied

#### 1. WebSocket Initial Config Status (ws.rs:1526)
**Problem**: New WebSocket clients always received `valid: true` status, even if the config was currently in an error state.

**Fix**: Changed from hardcoded `valid: true` to reading from stored `config_status_state`:
```rust
// Before:
let initial_config_status = ConfigStatusData {
    valid: true,
    error: None,
};

// After:
let initial_config_status = state.config_status.read().unwrap().clone();
```

**Impact**: New clients now see the current config error banner if one exists, fixing the "UI banner persists until operator fixes the file" requirement.

#### 2. Missing `roles` Field in `resolve_from_raw` (config_resolver.rs:1928)
**Problem**: The `roles` field was added to `ResolvedConfig` but not included in the `resolve_from_raw` function, causing compilation errors.

**Fix**: Added the `roles` field to the `resolve_from_raw` function with the same logic as `resolve`:
```rust
roles: if let Some(role_config) = yml_ref.and_then(|y| yaml_get_role_config(y)) {
    Resolved::new(
        role_config,
        ConfigSource::ConfigYml,
        "config.yml: roles".to_string(),
    )
} else {
    Resolved::new(
        RoleConfig::default(),
        ConfigSource::Default,
        "compiled default (no roles configured)".to_string(),
    )
},
```

## Test Coverage

### config.yml Tests (config_watcher.rs)

#### Integration Tests
- `test_edit_invalid_then_fix_cycle` - Full invalid→fix cycle preserves state
- `test_invalid_adapter_rejected` - Invalid agent.adapter value rejected
- `test_invalid_theme_rejected` - Invalid ui.theme value rejected
- `test_unknown_field_rejected` - Unknown top-level field rejected
- `test_empty_config_uses_defaults` - Missing config.yml uses all defaults
- `test_schema_version_integer_coerced_to_string` - Documents YAML coercion behavior

#### Type Validation Tests
- `test_invalid_metrics_port_type_rejected` - String instead of integer
- `test_invalid_audit_retention_days_type_rejected` - String instead of integer
- `test_invalid_reflection_threshold_type_rejected` - String instead of float
- `test_invalid_ui_archive_days_type_rejected` - String instead of integer
- `test_invalid_voice_max_seconds_type_rejected` - Boolean instead of integer
- `test_invalid_audit_hash_chain_type_rejected` - Integer instead of boolean
- `test_invalid_reflection_enabled_type_rejected` - String instead of boolean
- `test_invalid_metrics_enabled_type_rejected` - Integer instead of boolean

### projects.yaml Tests (projects.rs)

#### Integration Tests
- `test_edit_invalid_then_fix_cycle` - Full invalid→fix cycle preserves state
- `test_semantic_validation_rejects_nonexistent_path` - Missing workspace path rejected
- `test_validate_detects_duplicate_canonical_paths` - Symlink dedup detection
- `test_config_error_from_yaml_error` - YAML parse error structured details
- `test_schema_violation_surfaces_field_line_expected_got` - Structured error details

#### Semantic Validation Tests
- `test_projects_config_empty` - Empty config validation
- `test_canonical_cache_resolves_paths` - Symlink resolution
- `test_canonical_for_lookup` - Canonical path lookup
- `test_canonical_for_missing_returns_raw` - Missing cache fallback
- `test_reload_identical_content_no_delta` - No-op reload behavior

## Acceptance Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| Bad YAML rejected; old config continues running | ✅ | Implemented in both watchers with rollback |
| Bad schema rejected with specific error | ✅ | ConfigError with field, expected, got details |
| UI banner persists until operator fixes the file | ✅ | Fixed by reading from stored state |
| Test: every field has a validation error scenario | ⚠️ | Coverage exists but compilation issues prevent running tests |

## Architecture

### Event Flow

```
File Change Detected
    ↓
Debounce (2s for config.yml, 5s for projects.yaml)
    ↓
Parse YAML → structured
    ↓
Schema Validation (type checking)
    ↓
Semantic Validation (enum values, paths, etc.)
    ↓
[If Invalid]
    → ConfigError event
    → Metrics: hoop_config_reload_rejected_total.inc()
    → Audit log written
    → WebSocket broadcast with error details
    → config_status_state updated with error
    → OLD CONFIG CONTINUES RUNNING
    ↓
[If Valid]
    → ConfigReloaded event
    → Metrics: hoop_config_reload_success_total.inc()
    → Audit log written
    → WebSocket broadcast with valid status
    → config_status_state updated with valid status
    → NEW CONFIG APPLIED
```

### Data Structures

```rust
// Config error with structured details
pub struct ConfigError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub field: Option<String>,     // e.g. "agent.adapter"
    pub expected: Option<String>,   // e.g. "boolean"
    pub got: Option<String>,        // e.g. "string"
}

// WebSocket status data
pub struct ConfigStatusData {
    pub valid: bool,
    pub error: Option<ConfigErrorData>,
}

// Events
pub enum ConfigEvent {
    ConfigReloaded { config, prev_hash },
    ConfigError { error, prev_hash },
}
```

## Limitations & Known Issues

1. **YAML Type Coercion**: `serde_yaml` coerces some types automatically (e.g., integer to string for schema_version), which may accept configs that stricter JSON schema validation would reject.

2. **Compilation Issues**: There are unrelated compilation errors in other parts of the codebase that prevent running the full test suite.

3. **Test Coverage**: While comprehensive tests exist for the main validation scenarios, they cannot be run due to the compilation issues.

## References

- Plan reference: §17 Configuration hot-reload, §6 Phase 6 deliverable 2
- Implementation files:
  - `hoop-daemon/src/config_watcher.rs` - config.yml hot-reload
  - `hoop-daemon/src/projects.rs` - projects.yaml hot-reload
  - `hoop-daemon/src/config_resolver.rs` - Config validation
  - `hoop-daemon/src/ws.rs` - WebSocket event broadcasting
  - `hoop-daemon/src/lib.rs` - Event handling and state management
