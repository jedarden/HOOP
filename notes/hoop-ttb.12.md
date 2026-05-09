# §15 Backups & disaster recovery - Final Verification

## Status: ✅ COMPLETE

All closing criteria verified on 2026-05-09.

## Implementation Evidence

### 1. Backup runs on schedule; credentials validated ✅
- `hoop-daemon/src/backup_pipeline.rs:58-94` - `start_scheduler()` checks cron every 60s
- `hoop-daemon/src/backup_pipeline.rs:695-724` - `CronSchedule::matches()` implementation
- `hoop-daemon/src/backup.rs:83-126` - `BackupCredentials::from_env()` validates env vars
- `hoop-daemon/src/lib.rs:2575-2586` - Daemon starts backup scheduler on init

### 2. Restore from recent snapshot produces identical state ✅
- `hoop-cli/src/restore.rs:278-450` - `run_restore()` with full pipeline
- `hoop-daemon/src/snapshot_manifest.rs:68-78` - Version validation (rejects newer)
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - All 4 DR scenario tests
- Rollback safety: `restore.rs:420-433` auto-rollback on failure

### 3. Documentation covers all four DR scenarios ✅
- `docs/operations.md:515-818` - Complete disaster recovery runbook
  - Scenario 1: Disk death (30-60 min)
  - Scenario 2: fleet.db corruption (10-20 min)
  - Scenario 3: Accidental deletion (10-20 min)
  - Scenario 4: Host migration (1-2 hours)

### 4. age encryption works with key in env var ✅
- `hoop-daemon/src/backup_pipeline.rs:535-562` - `age_encrypt()` uses `HOOP_BACKUP_AGE_KEY`
- `hoop-cli/src/restore.rs:454-482` - `decrypt_with_age()` uses `HOOP_BACKUP_AGE_IDENTITY`

## Configuration

```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: false
```

## What Gets Backed Up
1. **fleet.db** - VACUUM INTO → zstd → age (optional) → S3
2. **Attachments** - Incremental sync via manifest diff
3. **Config files** - config.yml, projects.yaml
4. **manifest.json** - Uploaded LAST for completeness validation

## REST API
- `POST /api/backup/trigger` - Manual backup trigger

## CLI Commands
- `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>` - Restore from backup

## Test Coverage
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - 593 lines of DR scenario tests
- Unit tests in: `backup.rs`, `backup_pipeline.rs`, `snapshot_manifest.rs`, `config_backup.rs`, `restore.rs`

## Verification Date
2026-05-09
