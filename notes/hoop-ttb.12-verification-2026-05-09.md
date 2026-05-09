# §15 Backups & disaster recovery - Verification (2026-05-09)

## Verification Summary

Verified that §15 Backups & disaster recovery is fully implemented and all closing criteria are met.

## Closing Criteria Status ✅

### 1. Backup runs on schedule; credentials validated ✅

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs:55-94` - `start_scheduler()` checks cron every 60s
- `hoop-daemon/src/backup_pipeline.rs:693-750` - 5-field cron matcher
- `hoop-daemon/src/backup.rs:79-127` - `BackupCredentials::from_env()` validates env vars
- `hoop-daemon/src/backup.rs:173-254` - `load_backup_config()` validates URL and schedule

**Configuration:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false
```

**Credentials (env vars only):**
- `HOOP_BACKUP_ACCESS_KEY_ID`
- `HOOP_BACKUP_SECRET_ACCESS_KEY`
- `HOOP_BACKUP_AGE_KEY` (if encryption enabled)

### 2. Restore from recent snapshot produces identical state ✅

**Implementation:**
- `hoop-cli/src/restore.rs:278-458` - `run_restore()` full pipeline
- `hoop-daemon/src/snapshot_manifest.rs:62-79` - `validate()` rejects newer schema
- `hoop-daemon/src/fleet.rs:4887-4918` - `restore_and_migrate()` runs migrations
- `hoop-daemon/src/fleet.rs:522-587` - `verify_hash_chain()` integrity check

**Command:**
```bash
hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>
```

**Features:**
- Moves existing `~/.hoop/` aside for rollback
- Validates manifest before destructive action
- Decompresses and decrypts (if needed)
- Runs schema migrations
- Verifies audit hash chain
- Automatic rollback on failure

### 3. Documentation covers all four DR scenarios ✅

**Location:** `docs/operations.md:458-818`

**Scenarios covered:**
1. **Disk death** (lines 519-604) - Full host recovery from S3
2. **fleet.db corruption** (lines 606-657) - Database restore
3. **Accidental deletion** (lines 659-703) - `rm -rf ~/.hoop/` recovery
4. **Host migration** (lines 705-817) - New host setup and migration

### 4. age encryption works with key in env var ✅

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs:535-562` - `age_encrypt()` during backup
- `hoop-cli/src/restore.rs:461-489` - `decrypt_with_age()` during restore

**Configuration:**
- Set `encryption: true` in config.yml
- Set `HOOP_BACKUP_AGE_KEY` env var for encryption
- Set `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY` for decryption

## What Gets Backed Up

| Component | Method | Location |
|-----------|--------|----------|
| fleet.db | VACUUM INTO → zstd → age | `<prefix>/<snapshot>/fleet.db.zst[.age]` |
| Attachments | Incremental manifest sync | `<prefix>/<snapshot>/attachments/*.zst` |
| config.yml | Compressed backup | `<prefix>/<snapshot>/config.yml.zst` |
| projects.yaml | Compressed backup | `<prefix>/<snapshot>/projects.yaml.zst` |
| Manifest | JSON uploaded last | `<prefix>/<snapshot>/manifest.json` |

## Test Coverage

| Test | Location |
|------|----------|
| Backup pipeline | `backup_pipeline.rs:753-905` |
| Attachment sync | `attachment_sync.rs:340-784` |
| Manifest validation | `snapshot_manifest.rs:99-218` |
| Restore rollback | `restore.rs:607-886` |
| DR runbook | `tests/disaster_recovery_runbook.rs` |

## REST API

**Manual trigger:**
```bash
curl -X POST http://localhost:3000/api/backup/trigger
```

**Metrics:**
```
hoop_backup_last_success_timestamp
hoop_backup_last_size_bytes
hoop_backup_failures_total
```

## Conclusion

All §15 closing criteria are met. The backup and disaster recovery system is production-ready with:
- S3-compatible storage (B2, AWS S3, MinIO, Garage)
- Automated daily backups with configurable schedule
- Incremental attachment sync
- Optional age encryption
- Safe restore with automatic rollback
- Comprehensive DR documentation
