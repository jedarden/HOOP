# Backup Encryption Security Fix (bf-5lv71)

## Summary

Fixed a security vulnerability where the backup pipeline would silently upload unencrypted snapshots when age encryption failed (including the common case where `HOOP_BACKUP_AGE_KEY` was unset).

## Changes

### hoop-daemon/src/backup_pipeline.rs (lines 176-190)

**Before:**
```rust
Err(e) => {
    warn!("Age encryption failed, uploading unencrypted: {}", e);
    (compressed_path.clone(), false)
}
```

**After:**
```rust
Err(e) => {
    // Encryption is enabled but failed - fail the entire backup run
    // rather than silently uploading unencrypted data (security bug)
    error!("Age encryption failed (backup.enabled=true): {}", e);
    metrics::metrics().hoop_errors_total.inc(&["backup", "encryption_failure"]);
    bail!(
        "backup encryption enabled but age encryption failed: {}. \
         Set HOOP_BACKUP_AGE_KEY or disable encryption in config.",
        e
    );
}
```

### hoop-daemon/tests/backup_restore_cycle.rs

Added three test cases:
1. `backup_fails_when_encryption_enabled_but_key_missing` - Verifies run fails when encryption=true but no age key
2. `backup_succeeds_with_encryption_when_key_provided` - Verifies encrypted upload succeeds with valid key
3. `backup_succeeds_without_encryption_when_disabled` - Verifies plaintext upload when encryption=false

## Acceptance Criteria Met

- ✅ With `backup.encryption: true` and no age key, no object reaches S3 and an error is recorded
- ✅ Error is surfaced via the error path with `bail!()`
- ✅ Metrics `hoop_errors_total{subsystem="backup", error_type="encryption_failure"}` incremented
- ✅ Success timestamp `hoop_backup_last_success_timestamp` remains untouched (staleness alerts fire)
- ✅ Failed runs recorded in audit rows via `write_backup_failed()`
- ✅ Snapshot manifest is never written for aborted runs (bail before manifest creation)
- ✅ All three test cases implemented

## Security Impact

This fix prevents silent policy downgrade where an operator who explicitly opted into encryption (because the S3 endpoint is untrusted) would have their data uploaded in plaintext without their knowledge when the age key is missing or encryption fails for any reason.

## Plan Reference

- docs/plan/plan.md §15.3 (uploads optionally encrypted with age)
- docs/plan/plan.md §27 error taxonomy E6-003
