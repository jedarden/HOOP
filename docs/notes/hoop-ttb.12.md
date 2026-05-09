# §15 Backups & Disaster Recovery - Verification Summary

## Status: ✅ COMPLETE

All closing criteria for §15 have been verified:

### 1. Backup runs on schedule; credentials validated ✅
- **Implementation:** `hoop-daemon/src/backup_pipeline.rs` - `start_scheduler()` with cron matching
- **Implementation:** `hoop-daemon/src/backup.rs` - `load_backup_config()` validates credentials from env vars
- **Environment Variables:**
  - `HOOP_BACKUP_ACCESS_KEY_ID` - S3 access key
  - `HOOP_BACKUP_SECRET_ACCESS_KEY` - S3 secret key
  - `HOOP_BACKUP_AGE_KEY` - age encryption public key (when encryption enabled)

### 2. Restore from recent snapshot produces identical state (verified) ✅
- **Implementation:** `hoop-cli/src/restore.rs` - `run_restore()` with comprehensive restore logic
- **Tests:**
  - `test_rollback_restores_original` - Verifies rollback restores original state
  - `test_mid_failure_rollback_full_cycle` - Full lifecycle test
  - `test_successful_restore_then_cleanup_allows_daemon_start`
- **Integrity:** SHA-256 verification of fleet.db, audit hash chain verification

### 3. Documentation covers all four DR scenarios ✅
- **Location:** `docs/operations.md` (lines 458-832)
- **Scenarios covered:**
  1. Disk death (30-60 minutes)
  2. fleet.db corruption (10-20 minutes)
  3. Accidental deletion (10-20 minutes)
  4. Host migration (1-2 hours)
- **Each scenario includes:** Recovery procedure, pitfalls, verification steps

### 4. age encryption works with key in env var ✅
- **Backup:** `backup_pipeline.rs` - `age_encrypt()` uses `HOOP_BACKUP_AGE_KEY`
- **Restore:** `restore.rs` - `decrypt_with_age()` uses `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY`

## Backup Implementation Details

### What gets backed up:
1. **fleet.db** - Daily VACUUM INTO snapshot → zstd compression → optional age encryption → S3
2. **Attachments** - Incremental sync (only new/changed since last success)
3. **Config files** - config.yml and projects.yaml on every change + daily
4. **manifest.json** - Uploaded LAST (validates completeness)

### S3-compatible endpoint:
- Default: Backblaze B2 (matches ARMOR encrypted S3 proxy pattern)
- Also works with: AWS S3, MinIO, Garage, any S3 API

### Configuration:
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false              # set to true for age encryption
```

## Restore Implementation Details

### Command:
```bash
hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>
```

### Restore process:
1. Precondition check: daemon must not be running
2. Download and validate manifest (before any destructive action)
3. Move existing `~/.hoop/` aside for rollback
4. Download and restore fleet.db (with decompression/decryption)
5. Download and restore attachments
6. Restore config files (config.yml, projects.yaml)
7. Run schema migrations
8. Verify audit hash chain integrity
9. Cleanup rollback directories on success

### Rollback on failure:
- Automatic rollback to original state if any step fails
- Manual recovery path documented

## Test Coverage

### Unit tests:
- `backup.rs` - Config parsing, credential validation, cron validation
- `backup_pipeline.rs` - VACUUM INTO, zstd compression, cron matching, HMAC signing
- `snapshot_manifest.rs` - Manifest serialization, version validation
- `config_backup.rs` - Config file hashing and backup
- `restore.rs` - S3 URI parsing, manifest validation, rollback logic

### Integration tests:
- `disaster_recovery_runbook.rs` - All four DR scenarios with step-by-step validation

## References
- Plan §15: docs/plan/plan.md#L1202-L1263
- Operations: docs/operations.md#L458-L832
