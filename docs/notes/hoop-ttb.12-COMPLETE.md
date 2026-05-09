# §15 Backups & disaster recovery - COMPLETE

## Status: FULLY IMPLEMENTED AND VERIFIED ✅

All closing criteria met as of 2026-05-09.

## Implementation Summary

### Core Components

1. **Backup Configuration** (`hoop-daemon/src/backup.rs`)
   - S3-compatible endpoint configuration
   - Environment variable credentials (never in config files)
   - Configurable cron schedule and retention

2. **Backup Pipeline** (`hoop-daemon/src/backup_pipeline.rs`)
   - Daily VACUUM INTO snapshot
   - zstd compression + optional age encryption
   - AWS SigV4 S3 upload with retry logic

3. **Attachment Sync** (`hoop-daemon/src/attachment_sync.rs`)
   - Incremental sync via manifest-based diff
   - Tombstone retention for deleted files

4. **Config Backup** (`hoop-daemon/src/config_backup.rs`)
   - Backs up config.yml and projects.yaml
   - SHA-256 integrity verification

5. **Snapshot Manifest** (`hoop-daemon/src/snapshot_manifest.rs`)
   - Ties all backup pieces together
   - Schema version validation (§20.1)

6. **Restore Command** (`hoop-cli/src/restore.rs`)
   - `hoop restore --from s3://...`
   - Rollback protection on failure
   - Audit hash chain verification

### Documentation

- **`docs/operations.md`**: Complete DR runbook for all four scenarios
- **`docs/troubleshooting.md`**: Backup failure recovery procedures

### Test Coverage

- **`hoop-daemon/tests/disaster_recovery_runbook.rs`**: 18 integration tests
- **50+ unit tests** across all backup modules

## Closing Criteria Verification

✅ **Backup runs on schedule; credentials validated**
✅ **Restore from recent snapshot produces identical state (verified)**
✅ **Documentation covers all four DR scenarios**
✅ **age encryption works with key in env var**

## Child Tasks (All Closed)

- hoop-ttb.12.1: Backup config schema + env-var credential resolver
- hoop-ttb.12.2: Daily fleet.db snapshot pipeline
- hoop-ttb.12.3: Incremental attachments sync
- hoop-ttb.12.4: Backup manifest.json format + schema version pinning
- hoop-ttb.12.5: docs/operations.md — DR runbook
- hoop-ttb.12.6: hoop restore: newer-than-current snapshot rejection

## References

- Plan §15: Backups & disaster recovery
- Plan §20.1: Schema migration and version compatibility
