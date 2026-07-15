# config_watcher Production Signatures and Test Analysis

## Production Function Signatures

### `ConfigWatcher::reload_config` (lines 304-310)

**Current Signature:**
```rust
async fn reload_config(
    path: &Path,
    event_tx: tokio::sync::broadcast::Sender<ConfigEvent>,
    config: Arc<Mutex<ResolvedConfig>>,
    cli_overrides: CliOverrides,
    agent_config_changed_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<AgentConfigChanged>>>>,
)
```

**Parameters:**
1. `path: &Path` - Path to config.yml file
2. `event_tx: tokio::sync::broadcast::Sender<ConfigEvent>` - Event channel sender
3. `config: Arc<Mutex<ResolvedConfig>>` - Shared config state
4. `cli_overrides: CliOverrides` - CLI configuration overrides
5. `agent_config_changed_tx: Arc<Mutex<Option<tokio::sync::broadcast::Sender<AgentConfigChanged>>>>` - Channel for agent config changes (hoop-ttb.6.2.2)

---

## Analysis of 16 Test Call Sites

### Key Finding: **All 16 test call sites are ALREADY CORRECT**

All 16 locations pass the correct 5-argument signature required by the production function:

**Pattern used at all 16 locations:**
```rust
ConfigWatcher::reload_config(
    &config_path,                           // ✓ path: &Path
    event_tx.clone(),                       // ✓ event_tx: Sender<ConfigEvent>
    shared_config.clone(),                 // ✓ config: Arc<Mutex<ResolvedConfig>>
    cli_overrides.clone(),                  // ✓ cli_overrides: CliOverrides
    Arc::new(Mutex::new(None)),             // ✓ agent_config_changed_tx
)
.await;
```

---

## Detailed Mapping of All 16 Locations

### 1. Line 591 - `test_edit_invalid_then_fix_cycle`
```rust
// Line 591-597
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 2. Line 617 - `test_edit_invalid_then_fix_cycle` (second call)
```rust
// Line 618-624
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 3. Line 642 - `test_edit_invalid_then_fix_cycle` (third call)
```rust
// Line 644-650
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 4. Line 679 - `test_invalid_adapter_rejected`
```rust
// Line 682-688
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 5. Line 715 - `test_invalid_theme_rejected`
```rust
// Line 719-725
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 6. Line 751 - `test_unknown_field_rejected`
```rust
// Line 756-762
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 7. Line 787 - `test_empty_config_uses_defaults`
```rust
// Line 793-799
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 8. Line 832 - `test_schema_version_integer_coerced_to_string`
```rust
// Line 839-845
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 9. Line 873 - `test_invalid_metrics_port_type_rejected`
```rust
// Line 881-887
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 10. Line 915 - `test_invalid_audit_retention_days_type_rejected`
```rust
// Line 924-930
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 11. Line 956 - `test_invalid_reflection_threshold_type_rejected`
```rust
// Line 966-972
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 12. Line 997 - `test_invalid_ui_archive_days_type_rejected`
```rust
// Line 1008-1014
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 13. Line 1038 - `test_invalid_voice_max_seconds_type_rejected`
```rust
// Line 1050-1056
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 14. Line 1079 - `test_invalid_audit_hash_chain_type_rejected`
```rust
// Line 1092-1098
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 15. Line 1122 - `test_invalid_reflection_enabled_type_rejected`
```rust
// Line 1136-1142
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

### 16. Line 1165 - `test_invalid_metrics_enabled_type_rejected`
```rust
// Line 1180-1186
ConfigWatcher::reload_config(
    &config_path,                    // ✓ path
    event_tx.clone(),                // ✓ event_tx
    shared_config.clone(),           // ✓ config
    cli_overrides.clone(),            // ✓ cli_overrides
    Arc::new(Mutex::new(None)),      // ✓ agent_config_changed_tx
)
.await;
```

---

## Summary

### ✓ No Test Changes Required

All 16 test call sites are **already passing the correct 5-argument signature** that matches the production function:

| Parameter | Type | Test Value |
|-----------|------|------------|
| `path` | `&Path` | `&config_path` |
| `event_tx` | `tokio::sync::broadcast::Sender<ConfigEvent>` | `event_tx.clone()` |
| `config` | `Arc<Mutex<ResolvedConfig>>` | `shared_config.clone()` |
| `cli_overrides` | `CliOverrides` | `cli_overrides.clone()` |
| `agent_config_changed_tx` | `Arc<Mutex<Option<...>>>` | `Arc::new(Mutex::new(None))` |

The tests were correctly updated when the `agent_config_changed_tx` parameter was added (likely as part of hoop-ttb.6.2.2 implementation).

---

## Recommendation

**No changes needed to test code.** The bead's concern about "test updates" appears to be unfounded—all tests already match the production signature. If compilation errors exist, they may be due to other factors (e.g., import issues, type mismatches elsewhere) rather than incorrect function signatures at these 16 call sites.
