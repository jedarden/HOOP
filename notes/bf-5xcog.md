# Test Failure Findings — bead bf-5xcog

## Summary
**Failure Type**: Compilation failure (prevents test execution)  
**Test Command**: `nix-shell -p pkg-config openssl --run 'cargo test -p hoop-daemon --test epoch_sync_invariant'`  
**Test Name**: epoch_sync_invariant  
**Outcome**: Test compilation failed with 2 errors - test cannot execute

## Error Details

### Error 1: Field name mismatch (E0609)

**Location**: `hoop-daemon/tests/integration_harness.rs:602:50`

**Error Message**:
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

**Root Cause**: References `handle._temp_dir` but the `DaemonHandle` struct defines the field as `pub temp_dir: TempDir` (line 584)

**Fix**: Change `handle._temp_dir` to `handle.temp_dir`

---

### Error 2: Missing struct field (E0063)

**Location**: `hoop-daemon/tests/integration_harness.rs:269:5`

**Error Message**:
```
error[E0063]: missing field `workspace` in initializer of `Bead`
   --> hoop-daemon/tests/integration_harness.rs:269:5
    |
269 |     Bead {
    |     ^^^^ missing `workspace`
```

**Root Cause**: The `Bead` struct initializer in `create_mock_bead()` function is missing the `workspace` field. The `Bead` struct (defined in hoop-daemon/src/lib.rs:171-189) includes:
```rust
#[serde(skip_deserializing, default)]
pub workspace: String,
```

But the test's mock function only initializes:
- id, title, description, status, priority, issue_type
- created_at, updated_at, created_by, dependencies, project
- **Missing**: workspace

**Fix**: Add `workspace: String::default()` or `workspace: "test".to_string()` to the Bead initializer in `create_mock_bead()`

---

## Stack Trace
None - this is a compilation failure that prevents the test binary from being built. No runtime execution occurred.

## Additional Warnings (4 total, non-blocking)
1. Unused import: `futures_util::SinkExt` at epoch_sync_invariant.rs:14
2. Unused variable: `beads` at integration_harness.rs:754
3. Unused variable: `bead_id` at integration_harness.rs:1310
4. Unused variable: `i` at integration_harness.rs:1344

## Patterns and Hypotheses

### Pattern: Schema evolution drift
This is the second instance where test code has drifted from schema changes:
- Previous: `bf-28iku` documented similar compilation failures
- Current: Same errors persist, indicating fixes were not applied

### Hypothesis: Test code not updated after workspace field addition
The `workspace` field was added to the `Bead` struct (likely as part of project/workspace separation work in Phase 1), but the test harness mock function was never updated to include this field.

### Hypothesis: Naming convention inconsistency
The `_temp_dir` vs `temp_dir` inconsistency suggests either:
- A rename occurred without updating all references
- The underscore prefix was intended to mark it as unused/internal but was later made public

## Next Investigation Steps

1. **Immediate**: Fix the two compilation errors to enable test execution:
   - Change `handle._temp_dir` to `handle.temp_dir` at integration_harness.rs:602
   - Add `workspace: String::default()` to `create_mock_bead()` Bead initializer

2. **After compilation succeeds**: Run the test to capture any runtime failures:
   ```bash
   nix-shell -p pkg-config openssl --run 'cargo test -p hoop-daemon --test epoch_sync_invariant'
   ```

3. **Pattern investigation**: Check for other test code that may have similar drift from schema changes:
   - Search for other `Bead { ... }` initializers in test code
   - Verify all mock functions include required struct fields

4. **Prevention**: Consider adding:
   - Compilation tests to catch these errors earlier
   - Documentation of struct field requirements near mock functions

## Related Issues
- Parent bead: bf-5jpx7 (Investigate epoch sync test failure)
- Previous analysis: bf-28iku (Documented same compilation errors)
- Original capture: bf-a80p6 (Captured initial test output)

## Acceptance Criteria Status
- ✅ Parent bead identified: bf-5jpx7
- ✅ Error messages documented: E0609, E0063
- ✅ Location identified: integration_harness.rs:602, integration_harness.rs:269
- ✅ Failure type categorized: Compilation failure (schema drift)
- ✅ Stack trace excerpts: N/A (compilation failure)
- ✅ Patterns noted: Schema evolution drift, naming inconsistency
- ✅ Next steps suggested: Fix compilation, then test runtime behavior
