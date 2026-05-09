# §15 Backups & disaster recovery - Final Verification (2026-05-09)

## Summary

This document provides final verification that §15 Backups & disaster recovery (hoop-ttb.12) is fully implemented and all closing criteria are met.

## Implementation Status

### Core Components

| Component | Location | Status |
|-----------|----------|--------|
| Config parsing | `hoop-daemon/src/backup.rs` | ✅ Complete |
| Backup pipeline | `hoop-daemon/src/backup_pipeline.rs` | ✅ Complete |
| Attachment sync | `hoop-daemon/src/attachment_sync.rs` | ✅ Complete |
| Manifest schema | `hoop-daemon/src/snapshot_manifest.rs` | ✅ Complete |
| Config backup | `hoop-daemon/src/config_backup.rs` | ✅ Complete |
| Restore CLI | `hoop-cli/src/restore.rs` | ✅ Complete |
| REST API | `hoop-daemon/src/api_backup.rs` | ✅ Complete |

### Closing Criteria Verification

#### 1. Backup runs on schedule; credentials validated ✅

**Implementation details:**
- `load_backup_config()` in `backup.rs:173-254` validates config and credentials
- `BackupCredentials::from_env()` in `backup.rs:83-126` reads env vars only
- `start_scheduler()` in `backup_pipeline.rs:58-94` runs cron every 60s
- Cron matcher in `backup_pipeline.rs:695-750` supports 5-field cron syntax

**Environment variables:**
- `HOOP_BACKUP_ACCESS_KEY_ID` (required)
- `HOOP_BACKUP_SECRET_ACCESS_KEY` (required)
- `HOOP_BACKUP_AGE_KEY` (optional, for encryption)

**Configuration example:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-operator
  prefix: ex44/
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: false
```

#### 2. Restore from recent snapshot produces identical state ✅

**Implementation details:**
- `run_restore()` in `restore.rs:278-458` implements full restore pipeline
- `verify_sha256()` in `restore.rs:172-180` validates downloaded data
- `verify_hash_chain()` in `fleet.rs:522-587` verifies audit log integrity
- `restore_and_migrate()` in `fleet.rs:4887-4918` runs schema migrations

**Restore command:**
```bash
hoop restore --from s3://bucket/prefix/snapshot-id
```

**Safety features:**
- Precondition: daemon must not be running
- Moves existing `~/.hoop/` aside for rollback
- Validates manifest before any destructive action
- Rejects snapshots with newer schema version
- Automatic rollback on failure
- Cleanup of rollback directories on success

#### 3. Documentation covers all four DR scenarios ✅

**Location:** `docs/operations.md:458-831`

**Scenarios documented:**
1. **Disk death** (lines 519-604) - Complete host recovery procedure
2. **fleet.db corruption** (lines 606-657) - Database corruption recovery
3. **Accidental deletion** (lines 659-703) - Recovery from `rm -rf ~/.hoop/`
4. **Host migration** (lines 705-817) - New host setup and migration

Each scenario includes:
- Expected duration
- Step-by-step recovery procedure
- Common pitfalls and solutions
- Verification steps

#### 4. age encryption works with key in env var ✅

**Implementation details:**
- `age_encrypt()` in `backup_pipeline.rs:535-562` encrypts during backup
- `decrypt_with_age()` in `restore.rs:454-482` decrypts during restore

**Usage:**
- Set `encryption: true` in config.yml
- Set `HOOP_BACKUP_AGE_KEY` with age public key for encryption
- Set `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY` with age private key for decryption

## What Gets Backed Up

| Component | Method | S3 Key |
|-----------|--------|---------|
| fleet.db | VACUUM INTO → zstd → age | `<prefix>/<snapshot>/fleet.db.zst[.age]` |
| Attachments | Incremental sync | `<prefix>/<snapshot>/attachments/*.zst` |
| Attachment manifest | JSON | `<prefix>/<snapshot>/attachments.manifest.json` |
| config.yml | Compressed | `<prefix>/<snapshot>/config.yml.zst` |
| projects.yaml | Compressed | `<prefix>/<snapshot>/projects.yaml.zst` |
| Snapshot manifest | JSON (uploaded last) | `<prefix>/<snapshot>/manifest.json` |

## Test Coverage

| Test Suite | Location | Tests |
|------------|----------|-------|
| Backup config | `backup.rs:272-335` | 8 tests |
| Backup pipeline | `backup_pipeline.rs:753-905` | 10 tests |
| Attachment sync | `attachment_sync.rs:340-784` | 24 tests |
| Snapshot manifest | `snapshot_manifest.rs:99-218` | 9 tests |
| Restore | `restore.rs:524-886` | 13 tests |

## Metrics Exported

```
hoop_backup_last_success_timestamp - Unix timestamp of last successful backup
hoop_backup_last_size_bytes - Size of last successful backup
hoop_backup_failures_total - Total number of failed backup runs
hoop_backup_run_duration_seconds - Wall-clock duration of backup runs
```

## REST API

**Manual trigger endpoint:**
```
POST /api/backup/trigger
```

**Response:**
```json
{
  "status": "started",
  "message": "Backup started"
}
```

## Integration Points

### Daemon Startup
- `lib.rs:2558-2587` - Loads backup config and starts scheduler

### Shutdown Handling
- Scheduler subscribes to shutdown coordinator
- Clean shutdown when daemon stops

### Config Hot Reload
- Backup config changes require daemon restart
- Credentials are resolved once at startup

## Conclusion

All §15 closing criteria are met:
1. ✅ Backup runs on configurable schedule with validated credentials
2. ✅ Restore produces identical state with integrity verification
3. ✅ Documentation covers all four DR scenarios comprehensively
4. ✅ age encryption works with environment variable keys

The backup and disaster recovery system is production-ready and supports:
- S3-compatible storage (B2, AWS S3, MinIO, Garage, etc.)
- Automated daily backups with configurable cron schedule
- Incremental attachment sync for efficient storage
- Optional age encryption for sensitive data
- Safe restore with automatic rollback on failure
- Comprehensive disaster recovery documentation
