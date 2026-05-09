# §15 Backups & Disaster Recovery - Verification Summary

## Closing Criteria Status

All closing criteria for §15 (Backups & disaster recovery) are **MET**:

### 1. ✅ Backup runs on schedule; credentials validated

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs` - BackupPipeline with `start_scheduler()` method
- `hoop-daemon/src/backup.rs` - `load_backup_config()` validates credentials from env vars
- `hoop-daemon/src/lib.rs:2576-2588` - Scheduler integration

**Configuration:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false              # set to true for age encryption
```

**Credentials (environment variables):**
```bash
export HOOP_BACKUP_ACCESS_KEY_ID="your-access-key"
export HOOP_BACKUP_SECRET_ACCESS_KEY="your-secret-key"
# If encryption is enabled:
export HOOP_BACKUP_AGE_KEY="age1...your-public-key"
```

**Tests:**
- `hoop-daemon/tests/backup_restore_cycle.rs::backup_credentials_validation`
- `hoop-daemon/tests/backup_restore_cycle.rs::backup_scheduler_runs_on_cron_schedule`

### 2. ✅ Restore from recent snapshot produces identical state (verified)

**Implementation:**
- `hoop-cli/src/restore.rs` - Full restore implementation with `run_restore()`
- `hoop-daemon/src/snapshot_manifest.rs` - Manifest validation and schema version checking
- `hoop-daemon/src/fleet.rs` - `restore_and_migrate()` for schema migrations

**Restore command:**
```bash
hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>
```

**Tests:**
- `hoop-daemon/tests/backup_restore_cycle.rs::backup_restore_cycle_produces_identical_state`
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - All four DR scenarios covered

### 3. ✅ Documentation covers all four DR scenarios

**Documentation:**
- `docs/operations.md:458-831` - Comprehensive disaster recovery documentation

**Four scenarios covered:**
1. **Disk death** (lines 519-604) - Restore to new host from S3 snapshot
2. **fleet.db corruption** (lines 606-658) - Restore from backup with integrity check
3. **Accidental deletion** (lines 660-704) - Recovery after `rm -rf ~/.hoop/`
4. **Host migration** (lines 706-818) - Full migration procedure

**Each scenario includes:**
- Expected duration
- Step-by-step recovery procedure
- Common pitfalls and how to avoid them
- Verification steps

### 4. ✅ age encryption works with key in env var

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs:535-562` - `age_encrypt()` reads from `HOOP_BACKUP_AGE_KEY`
- `hoop-cli/src/restore.rs:454-482` - `decrypt_with_age()` reads from `HOOP_BACKUP_AGE_IDENTITY`

**Encryption flow:**
1. Backup reads `HOOP_BACKUP_AGE_KEY` (public key) for encryption
2. Restore reads `HOOP_BACKUP_AGE_IDENTITY` (private key file) for decryption
3. Uses `age` CLI tool for actual encryption/decryption operations

**Test:**
- `hoop-daemon/tests/backup_restore_cycle.rs::age_encryption_with_env_key`

## What Gets Backed Up

Per §15.3:
- **`fleet.db`** - SQLite database via `VACUUM INTO` snapshot
- **Attachments** - Incremental sync (new/changed files only)
- **Config files** - `config.yml`, `projects.yaml`

## Snapshot Structure

Each snapshot produces:
```
<prefix>/<snapshot-id>/
├── manifest.json              # Uploaded LAST (validates completeness)
├── fleet.db.zst              # Compressed database
├── fleet.db.zst.age          # Age-encrypted (if encryption enabled)
├── config.yml.zst            # Compressed config
├── projects.yaml.zst         # Compressed projects registry
├── attachments.manifest.json # Attachment inventory
└── attachments/              # New/changed attachments (incremental)
    └── *.zst
```

## Metrics

Per §16.6, backup-related metrics are exposed:
- `hoop_backup_last_success_timestamp` - Unix timestamp of last success
- `hoop_backup_last_size_bytes` - Size of last backup
- `hoop_backup_failures_total` - Total failure count
- `hoop_backup_run_duration_seconds` - Backup run duration histogram

## Test Coverage

**Integration tests:**
- `hoop-daemon/tests/backup_restore_cycle.rs` - Backup/restore cycle, credentials, age encryption
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - All four DR scenarios

**Unit tests (in module files):**
- `hoop-daemon/src/backup.rs` - Config parsing, credential validation
- `hoop-daemon/src/backup_pipeline.rs` - VACUUM INTO, zstd compression, cron parsing
- `hoop-daemon/src/attachment_sync.rs` - Incremental sync diff engine
- `hoop-daemon/src/snapshot_manifest.rs` - Manifest validation, version checks
- `hoop-cli/src/restore.rs` - S3 URI parsing, rollback logic, manifest validation

## Status: COMPLETE ✓

All §15 deliverables implemented, tested, and documented.
