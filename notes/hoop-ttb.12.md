# §15 Backups & disaster recovery - Implementation verification

## Closing criteria verification

### 1. Backup runs on schedule; credentials validated ✅

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs`: `BackupPipeline::start_scheduler()` - runs on cron schedule
- `hoop-daemon/src/backup.rs`: `load_backup_config()` - validates credentials from environment
- `hoop-daemon/src/backup.rs`: `BackupCredentials::from_env()` - returns `None` if credentials missing

**Evidence:**
- Daily cron scheduler checks every 60 seconds (line 69)
- Credentials read from `HOOP_BACKUP_ACCESS_KEY_ID`, `HOOP_BACKUP_SECRET_ACCESS_KEY`
- Age key from `HOOP_BACKUP_AGE_KEY` when encryption enabled
- Warning logs when credentials missing

### 2. Restore from recent snapshot produces identical state (verified) ✅

**Implementation:**
- `hoop-cli/src/restore.rs`: `run_restore()` - full restore implementation
- `hoop-daemon/tests/backup_restore_cycle.rs`: Integration test verifying identical state

**Evidence:**
- Manifest validation before any destructive action
- SHA-256 integrity checks on downloaded files
- Atomic rollback on failure
- Test compares SHA-256 hashes before/after restore

### 3. Documentation covers all four DR scenarios ✅

**Location:** `docs/operations.md` lines 519-818

**Scenarios covered:**
1. **Disk death** (lines 521-604)
2. **fleet.db corruption** (lines 606-657)
3. **Accidental deletion** (lines 659-703)
4. **Host migration** (lines 705-817)

Each scenario includes:
- Situation description
- Expected duration
- Step-by-step recovery procedure
- Pitfalls section

### 4. age encryption works with key in env var ✅

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs`: `age_encrypt()` (lines 534-562)
- `hoop-cli/src/restore.rs`: `decrypt_with_age()` (lines 454-482)
- `hoop-daemon/tests/backup_restore_cycle.rs`: `age_encryption_with_env_key()` test (lines 189-274)

**Evidence:**
- Encryption uses `HOOP_BACKUP_AGE_KEY` env var
- Decryption uses `HOOP_BACKUP_AGE_IDENTITY` env var
- Test verifies encrypt/decrypt roundtrip

## What gets backed up

Per `backup_pipeline.rs`:
- `fleet.db` - SQLite VACUUM INTO snapshot → zstd → age (optional)
- Attachments - incremental sync (only new/changed since last success)
- Config files - `config.yml`, `projects.yaml` on every change plus daily
- `manifest.json` - schema version + piece list (uploaded last)

## Restore flow

Per `restore.rs`:
1. Precondition check: daemon must not be running
2. Parse S3 URI and load config
3. Download and parse manifest
4. Validate manifest (schema version check)
5. Move existing `~/.hoop/` aside for rollback
6. Download and restore `fleet.db` (compressed, optionally encrypted)
7. Download and restore attachments
8. Restore config files
9. Run schema migrations
10. Verify audit hash chain integrity
11. Clean up rollback directories

## Metrics exposed

Per `metrics.rs` lines 765-771:
- `hoop_backup_last_success_timestamp` (gauge)
- `hoop_backup_last_size_bytes` (gauge)
- `hoop_backup_failures_total` (counter)
- `hoop_backup_run_duration_seconds` (histogram)

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

## Status: COMPLETE

All closing criteria for §15 (Backups & disaster recovery) have been verified through:
1. Code review of implementation files
2. Test verification in `backup_restore_cycle.rs`
3. Documentation review in `docs/operations.md`
4. Metrics integration verified in `metrics.rs`
