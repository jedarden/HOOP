# §15 Backups & disaster recovery - Verification Summary

## Implementation Status: COMPLETE

All §15 requirements have been implemented and verified.

## Components Implemented

### 1. Configuration (`hoop-daemon/src/backup.rs`)
- ✅ `BackupFileConfig` - S3 endpoint, bucket, prefix, schedule, retention, encryption
- ✅ `BackupCredentials` - S3 access keys from env vars (never in config)
- ✅ `BackupState` - NotConfigured, Disabled, Ready states
- ✅ `load_backup_config()` - Validates config and resolves credentials
- ✅ Environment variable credentials: `HOOP_BACKUP_ACCESS_KEY_ID`, `HOUP_BACKUP_SECRET_ACCESS_KEY`, `HOOP_BACKUP_AGE_KEY`

### 2. Backup Pipeline (`hoop-daemon/src/backup_pipeline.rs`)
- ✅ `VACUUM INTO` for fleet.db snapshots (SQLite online backup)
- ✅ zstd compression (level 3)
- ✅ age encryption (optional, via `age --encrypt`)
- ✅ S3 upload with AWS SigV4 signing
- ✅ Exponential backoff retry (max 3 attempts, 2s → 60s)
- ✅ Cron scheduler (5-field, checks every 60s)
- ✅ Manual trigger via API (`POST /api/backup/trigger`)
- ✅ Metrics: `hoop_backup_last_success_timestamp`, `hoop_backup_last_size_bytes`, `hoop_backup_failures_total`, `hoop_backup_run_duration_seconds`
- ✅ Audit logging: `BackupStarted`, `BackupFinished`, `BackupFailed` events

### 3. Attachment Sync (`hoop-daemon/src/attachment_sync.rs`)
- ✅ Manifest-based incremental sync
- ✅ SHA-256 file hashing for change detection
- ✅ mtime+size fast-path optimization
- ✅ Tombstone tracking for deleted files
- ✅ Configurable retention period
- ✅ Atomic manifest writes (tmp + rename)
- ✅ Separate namespaces: `stitch/<id>/<file>` and `bead/<id>/<file>`

### 4. Config Backup (`hoop-daemon/src/config_backup.rs`)
- ✅ `config.yml` backup (SHA-256 + size tracking)
- ✅ `projects.yaml` backup (SHA-256 + size tracking)
- ✅ Uploads compressed files to S3
- ✅ Preserved from rollback during restore

### 5. Snapshot Manifest (`hoop-daemon/src/snapshot_manifest.rs`)
- ✅ `snapshot_id` (ISO 8601 timestamp)
- ✅ `schema_version` (for migration validation)
- ✅ `created_at` timestamp
- ✅ `fleet_db_key`, `fleet_db_sha256`, `fleet_db_size`
- ✅ `attachments_manifest_key`
- ✅ `encryption` mode ("none" or "age")
- ✅ `hoop_version`
- ✅ `final_audit_hash` (hash chain integrity anchor)
- ✅ `config_backup` metadata
- ✅ `validate()` - rejects newer schema versions (§20.1)

### 6. Restore CLI (`hoop-cli/src/restore.rs`)
- ✅ `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
- ✅ Daemon running check (precondition)
- ✅ S3 download with SigV4 signing
- ✅ Manifest validation (before destructive action)
- ✅ SHA-256 integrity verification
- ✅ zstd decompression
- ✅ age decryption (optional)
- ✅ Rollback mechanism on failure
- ✅ Schema migrations (`restore_and_migrate()`)
- ✅ Audit hash chain verification
- ✅ Cleanup of rollback dirs on success

### 7. REST API (`hoop-daemon/src/api_backup.rs`)
- ✅ `POST /api/backup/trigger` - Manual backup trigger

### 8. Metrics (`hoop-daemon/src/metrics.rs`)
- ✅ `hoop_backup_last_success_timestamp` (Gauge)
- ✅ `hoop_backup_last_size_bytes` (Gauge)
- ✅ `hoop_backup_failures_total` (Counter)
- ✅ `hoop_backup_run_duration_seconds` (Histogram)

### 9. Tests (`hoop-daemon/tests/disaster_recovery_runbook.rs`)
- ✅ Scenario 1: Disk death - restore to fresh state
- ✅ Scenario 2: fleet.db corruption - integrity check detection
- ✅ Scenario 3: Accidental deletion - recovery after `rm -rf ~/.hoop/`
- ✅ Scenario 4: Host migration - project path updates
- ✅ Rollback mechanism tests
- ✅ Duration bound tests
- ✅ Pitfall detection tests
- ✅ Manifest validation tests

### 10. Documentation (`docs/operations.md`)
- ✅ Backup configuration section
- ✅ Manual backup trigger instructions
- ✅ All four disaster recovery scenarios documented
- ✅ Pitfalls and error handling documented
- ✅ Environment variable requirements documented

## What Gets Backed Up

| Component | Method | Frequency | Location |
|-----------|--------|-----------|----------|
| `fleet.db` | `VACUUM INTO` → zstd → age (optional) | Daily (configurable) | `{prefix}/{snapshot_id}/fleet.db.zst[.age]` |
| Attachments | Incremental sync (new/changed only) | Daily | `{prefix}/{snapshot_id}/attachments/*.zst` |
| `config.yml` | Compressed upload | On change + daily | `{prefix}/{snapshot_id}/config.yml.zst` |
| `projects.yaml` | Compressed upload | On change + daily | `{prefix}/{snapshot_id}/projects.yaml.zst` |
| Manifest | JSON (uploaded last) | Every run | `{prefix}/{snapshot_id}/manifest.json` |

## Disaster Scenarios Covered

1. **Disk death** - Restore to new host from latest S3 snapshot (30-60 min)
2. **fleet.db corruption** - Restore from backup, lose at most one day
3. **Accidental deletion** - Same recovery path as corruption
4. **Host migration** - Fresh HOOP install + restore + project workspace migration

## Configuration Example

```yaml
# ~/.hoop/config.yml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false             # set to true for age encryption
```

## Environment Variables (Credentials)

```bash
export HOOP_BACKUP_ACCESS_KEY_ID="your-access-key"
export HOOP_BACKUP_SECRET_ACCESS_KEY="your-secret-key"
# Optional for encryption:
export HOOP_BACKUP_AGE_KEY="age-recipient-public-key"
# For restore with encrypted backups:
export HOOP_BACKUP_AGE_IDENTITY="/path/to/age-private-key"
```

## Integration Points

1. **Daemon startup** (`hoop-daemon/src/lib.rs`):
   - Loads backup config via `load_backup_config()`
   - Starts cron scheduler when `BackupState::Ready`
   - Stores `BackupPipeline` in `DaemonState.backup_runner`

2. **API integration** (`hoop-daemon/src/lib.rs`):
   - `api_backup::router()` mounted at `/api/backup/trigger`

3. **CLI integration** (`hoop-cli/src/main.rs`):
   - `hoop restore --from s3://...` command

## Closing Criteria Verification

- ✅ Backup runs on schedule; credentials validated
- ✅ Restore from recent snapshot produces identical state (verified in tests)
- ✅ Documentation covers all four DR scenarios
- ✅ age encryption works with key in env var

## Files Modified/Created

### Core Implementation
- `hoop-daemon/src/backup.rs` - Config and credentials
- `hoop-daemon/src/backup_pipeline.rs` - Full backup pipeline
- `hoop-daemon/src/attachment_sync.rs` - Incremental attachment sync
- `hoop-daemon/src/config_backup.rs` - Config file backup
- `hoop-daemon/src/snapshot_manifest.rs` - Snapshot manifest

### API & CLI
- `hoop-daemon/src/api_backup.rs` - REST API endpoint
- `hoop-cli/src/restore.rs` - CLI restore command

### Tests
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - DR scenario tests

### Schema
- `hoop-schema/schemas/backup_config.json` - Backup config schema

### Documentation
- `docs/operations.md` - Full backup & DR documentation

## Notes

- The backup subsystem is **fully optional** - HOOP runs without it if not configured
- Credentials are **never** stored in `config.yml` or written to logs
- Restore validates manifest **before** any destructive action (move_aside_for_rollback)
- Rollback is automatic on any failure after the destructive rename
- Attachment sync uses tombstones to track deletions for N days (configurable)
