# §15 Backups & disaster recovery - Verification Complete

## Date: 2026-05-09

## Summary

Verified that the backup and disaster recovery system (§15) is **fully implemented** and all closing criteria are met.

## Implementation Components

### Core Backup System

| Component | File | Purpose |
|-----------|------|---------|
| Config parser | `hoop-daemon/src/backup.rs` | Reads `backup:` section from `~/.hoop/config.yml`, resolves credentials from env vars |
| Backup pipeline | `hoop-daemon/src/backup_pipeline.rs` | VACUUM INTO → zstd → age (optional) → S3 upload with cron scheduler |
| Attachment sync | `hoop-daemon/src/attachment_sync.rs` | Incremental attachment sync with manifest-based diff engine |
| Config backup | `hoop-daemon/src/config_backup.rs` | Backs up config.yml and projects.yaml on every change |
| Snapshot manifest | `hoop-daemon/src/snapshot_manifest.rs` | Ties all backup pieces together with schema version |
| Restore CLI | `hoop-cli/src/restore.rs` | `hoop restore --from s3://...` with rollback and migration |
| Backup CLI | `hoop-cli/src/backup.rs` | `hoop backup trigger` and `hoop backup status` |
| API endpoint | `hoop-daemon/src/api_backup.rs` | REST API for manual backup trigger |

### Closing Criteria Verification

#### 1. Backup runs on schedule; credentials validated ✅

**Configuration:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false            # or true for age encryption
```

**Credentials (env vars only):**
- `HOOP_BACKUP_ACCESS_KEY_ID`
- `HOOP_BACKUP_SECRET_ACCESS_KEY`
- `HOOP_BACKUP_AGE_KEY` (when encryption enabled)

**Scheduler:**
- Checks cron schedule every 60 seconds
- Tracks last run date to prevent duplicate runs
- Exponential backoff retry (max 3 attempts)

**Metrics:**
- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size in bytes of last successful backup
- `hoop_backup_failures_total` - Total number of backup failures
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

#### 2. Restore from recent snapshot produces identical state (verified) ✅

**Restore process:**
1. Precondition check: daemon must not be running
2. Download and validate manifest.json
3. Validate schema version (rejects newer snapshots)
4. Move existing `~/.hoop/` aside for rollback
5. Download fleet.db, attachments, config files
6. Run schema migrations
7. Verify audit hash chain integrity
8. Cleanup rollback dirs on success

**Tests:**
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - 18 tests covering:
  - All 4 disaster scenarios
  - Rollback mechanism
  - Pitfall detection
  - Version mismatch rejection

#### 3. Documentation covers all four DR scenarios ✅

**File:** `docs/operations.md` (lines 458-831)

| Scenario | Section | Duration |
|----------|---------|----------|
| Disk death | 519-604 | 30-60 minutes |
| fleet.db corruption | 606-658 | 10-20 minutes |
| Accidental deletion | 659-704 | 10-20 minutes |
| Host migration | 705-831 | 1-2 hours |

Each scenario includes:
- Situation description
- Expected duration
- Step-by-step recovery procedure
- Common pitfalls

#### 4. age encryption works with key in env var ✅

**Encryption (backup):**
- `backup_pipeline.rs::age_encrypt()` - spawns `age --encrypt --recipient $HOOP_BACKUP_AGE_KEY`
- Output: `fleet.db.zst.age`

**Decryption (restore):**
- `restore.rs::decrypt_with_age()` - spawns `age --decrypt --identity $HOOP_BACKUP_AGE_IDENTITY`
- Supports both `HOOP_BACKUP_AGE_IDENTITY` and `AGE_IDENTITY` env vars

### What Gets Backed Up

- `fleet.db` - SQLite `VACUUM INTO` snapshot (daily)
- `attachments/` - Incremental sync (only new/changed files)
- `config.yml` - On every change plus daily
- `projects.yaml` - On every change plus daily

Each backup produces a snapshot with:
- `manifest.json` - Uploaded last (prevents partial snapshots)
- `fleet.db.zst` or `fleet.db.zst.age` - Compressed (and optionally encrypted) database
- `attachments.manifest.json` - Attachment inventory
- `attachments/*.zst` - New/changed attachment files
- `config.yml.zst` - Config backup
- `projects.yaml.zst` - Project registry backup

### What's Explicitly NOT Backed Up

- Bead state (`.beads/` in each workspace - `br`'s job)
- NEEDLE worker state (separate system)
- CLI session files (each CLI owns these)
- Git worktree state (git's job)

## Integration Points

**Daemon:**
- `lib.rs` - `backup_runner: Option<Arc<backup_pipeline::BackupPipeline>>`
- Scheduler started on daemon init when `BackupState::Ready`
- Broadcast channel for shutdown coordination

**CLI:**
```bash
hoop backup trigger      # Manual backup trigger
hoop backup status       # Show configuration and last run status
hoop restore --from s3://...  # Restore from snapshot
```

**API:**
```
POST /api/backup/trigger   # Manual backup trigger
GET /metrics               # Prometheus metrics including backup status
```

## Verification Status

✅ All closing criteria met
✅ All 4 DR scenarios documented with runbooks
✅ Comprehensive test coverage (18 tests)
✅ age encryption with env var key
✅ S3-compatible storage (B2, AWS S3, MinIO, Garage, etc.)
✅ Configurable retention with tombstone pruning
✅ Idempotent restore with automatic rollback
