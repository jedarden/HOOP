# Epoch Sync Test Failure Analysis

**Date**: 2026-07-04  
**Test Target**: `hoop-daemon/tests/epoch_sync_invariant.rs`  
**Bead**: bf-132m2

## Failure Type Classification

**Compilation Failure** — The test did not execute at all. The Rust compiler failed to build the test binary due to two structural code errors:

1. **Field name mismatch** (E0609): `handle._temp_dir` → `handle.temp_dir`
2. **Missing required field** (E0063): `Bead` struct missing `workspace` field

## Specific Compilation Errors

### Error 1: Field Access Mismatch
**Location**: `hoop-daemon/tests/integration_harness.rs:602`

```
error[E0609]: no field `_temp_dir` on type `DaemonHandle`
   --> hoop-daemon/tests/integration_harness.rs:602:50
    |
602 |     Ok((base_url, handle.shutdown_notify, handle._temp_dir))
    |                                                  ^^^^^^^^^ unknown field
    |
help: a field with a similar name exists
    |
602 -     Ok((base_url, handle.shutdown_notify, handle._temp_dir))
602 +     Ok((base_url, handle.shutdown_notify, handle.temp_dir))
```

**Root Cause**: The `DaemonHandle` struct was refactored to rename the field from `_temp_dir` (private convention) to `temp_dir` (public). The test code still references the old private field name.

**Current Struct** (line 582-585 in integration_harness.rs):
```rust
pub struct DaemonHandle {
    shutdown_notify: Arc<tokio::sync::Notify>,
    pub temp_dir: TempDir,  // Now public
}
```

### Error 2: Missing Struct Field
**Location**: `hoop-daemon/tests/integration_harness.rs:269`

```
error[E0063]: missing field `workspace` in initializer of `Bead`
   --> hoop-daemon/tests/integration_harness.rs:269:5
    |
269 |     Bead {
    |     ^^^^ missing `workspace`
```

**Root Cause**: The `Bead` struct (hoop-daemon/src/lib.rs:171-189) now includes a `workspace: String` field (added for HOOP's multi-workspace architecture), but the test helper `create_mock_bead()` does not initialize it.

**Current Bead Struct** (hoop-daemon/src/lib.rs:171-189):
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bead {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: BeadStatus,
    pub priority: i64,
    pub issue_type: BeadType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub dependencies: Vec<String>,
    /// Project name assigned by HOOP at load time — not stored in issues.jsonl
    #[serde(skip_deserializing, default)]
    pub project: String,
    /// Workspace path assigned by HOOP at load time — not stored in issues.jsonl
    #[serde(skip_deserializing, default)]
    pub workspace: String,  // ← Missing from test code
}
```

**Problematic Test Code** (integration_harness.rs:267-281):
```rust
pub fn create_mock_bead(id: &str, title: &str, status: BeadStatus, project: &str) -> Bead {
    use chrono::Utc;
    Bead {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        status,
        priority: 0,
        issue_type: BeadType::Task,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: "test".to_string(),
        dependencies: vec![],
        project: project.to_string(),
        // Missing: workspace field
    }
}
```

## Root Cause Hypothesis

**Structural API drift**: The `Bead` and `DaemonHandle` data structures evolved during Phase 0-1 implementation to support HOOP's multi-workspace architecture. Test infrastructure (integration_harness.rs) was not updated synchronously with these schema changes.

The `workspace` field was added to `Bead` to track which workspace (repo) a bead belongs to, critical for HOOP's project → workspace mapping. The `temp_dir` field on `DaemonHandle` was made public to allow test helpers to access the temporary directory.

Both are legitimate API changes; the test code simply needs updating to match.

## Next Steps Suggested

### Immediate Fixes (for bead `bf-132m2`)

1. **Fix `create_mock_bead()`** — Add `workspace` field:
   ```rust
   pub fn create_mock_bead(id: &str, title: &str, status: BeadStatus, project: &str, workspace: &str) -> Bead {
       // ... existing fields ...
       project: project.to_string(),
       workspace: workspace.to_string(),  // ← Add this
   }
   ```
   And update all call sites (grep: `create_mock_bead`).

2. **Fix `spawn_test_daemon()`** — Change field access:
   ```rust
   Ok((base_url, handle.shutdown_notify, handle.temp_dir))  // ← was _temp_dir
   ```

### Secondary Issues

The compiler also emitted warnings for unused code:
- Unused imports: `futures_util::SinkExt` in epoch_sync_invariant.rs
- Unused variables in integration_harness.rs: `beads`, `bead_id`, loop counter `i`
- Dead code: `openapi_router()`, `load_hoop_config()`, `check_and_emit_capacity_alert()`, various unused fields

These do not block compilation but suggest incomplete refactoring or WIP code.

### Verification

After fixes:
1. `cargo test -p hoop-daemon --test epoch_sync_invariant -- --nocapture --test-threads=1`
2. Check that the test compiles and executes (or fails at runtime with actual test assertions, not compilation errors)

### Scope Note

This bead is **analysis-only**. Fixing the compilation errors belongs to a separate implementation bead. This document captures the diagnosis for the fix to proceed cleanly.
