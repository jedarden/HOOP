# Backup & Disaster Recovery Implementation Verification

**Plan Section:** §15
**Date:** 2026-05-09
**Status:** ✅ COMPLETE

## Closing Criteria Verification

### 1. Backup runs on schedule; credentials validated ✅

**Implementation:**
- `hoop-daemon/src/backup.rs` - Configuration loader and credential validator
  - Reads `backup:` section from `~/.hoop/config.yml`
  - Validates credentials from `HOOP_BACKUP_ACCESS_KEY_ID` and `HOOP_BACKUP_SECRET_ACCESS_KEY`
  - Validates age key from `HOOP_BACKUP_AGE_KEY` when encryption is enabled
  - Returns `BackupState::Ready` when fully configured

- `hoop-daemon/src/backup_pipeline.rs` - Backup pipeline with scheduler
  - `start_scheduler()` method spawns cron-based task checking every 60 seconds
  - Uses 5-field cron parser (minute hour dom mon dow)
  - Runs at configured schedule (default: `0 4 * * *` for daily 04:00)
  - Integrated into daemon startup at `lib.rs:2590-2593`

**Configuration Example:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: false
```

**Environment Variables Required:**
```bash
export HOOP_BACKUP_ACCESS_KEY_ID="your-access-key"
export HOOP_BACKUP_SECRET_ACCESS_KEY="your-secret-key"
# If encryption enabled:
export HOOP_BACKUP_AGE_KEY="age1...your-public-key"
```

### 2. Restore from recent snapshot produces identical state (verified) ✅

**Implementation:**
- `hoop-cli/src/restore.rs` - Complete restore implementation
  - Parses S3 URI: `s3://bucket/prefix/snapshot-id`
  - Downloads and validates manifest.json first
  - Validates schema version (rejects newer versions)
  - Moves existing `~/.hoop/` aside to `~/.hoop.rollback.YYYYMMDDTHHMMSSZ`
  - Restores fleet.db (with SHA-256 integrity check)
  - Restores attachments from incremental manifest
  - Restores config.yml and projects.yaml
  - Runs schema migrations on restored database
  - Verifies audit hash chain integrity
  - Automatic rollback on any failure

**Verification Steps:**
```bash
# List snapshots
aws s3 ls s3://hoop-backups-<operator>/ex44/

# Restore from snapshot
hoop restore --from s3://hoop-backups-<operator>/ex44/20240615T040000Z

# Verify database integrity
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"
```

### 3. Documentation covers all four DR scenarios ✅

**Location:** `docs/operations.md` sections 458-831

**Scenarios Covered:**

#### Scenario 1: Disk death (lines 519-604)
- Expected duration: 30-60 minutes
- Procedure: Provision new host → Install HOOP → Set credentials → Restore
- Includes pitfalls and verification steps

#### Scenario 2: fleet.db corruption (lines 606-657)
- Expected duration: 10-20 minutes
- Procedure: Confirm corruption → Preserve corrupted DB → List snapshots → Restore
- Notes data loss window (up to 24 hours) and attachment desync considerations

#### Scenario 3: Accidental deletion (lines 659-703)
- Expected duration: 10-20 minutes
- Procedure: Stop daemon → Set credentials → Restore → Verify
- Covers complete `rm -rf ~/.hoop/` scenario

#### Scenario 4: Host migration (lines 705-817)
- Expected duration: 1-2 hours
- Procedure: Final backup on old host → Stop services → Prepare new host → Install HOOP → Restore workspaces → Restore HOOP state
- Notes that project code and `.beads/` must be migrated separately

### 4. age encryption works with key in env var ✅

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs:age_encrypt()`
  - Spawns `age --encrypt --recipient $HOOP_BACKUP_AGE_KEY`
  - Produces `.age` file extension
  - Falls back to unencrypted upload on failure (with warning)
  
- `hoop-cli/src/restore.rs:decrypt_with_age()`
  - Spawns `age --decrypt --identity $HOUP_BACKUP_AGE_IDENTITY`
  - Required for encrypted snapshots

**Environment Variables:**
- Backup: `HOOP_BACKUP_AGE_KEY` - age public key (recipient)
- Restore: `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY` - age private key file path

## Snapshot Structure

Each snapshot produces:
```
s3://bucket/prefix/<snapshot-id>/
├── fleet.db.zst              (or fleet.db.zst.age if encrypted)
├── attachments.manifest.json
├── attachments/<path>.zst     (incremental: only new/changed)
├── config.yml.zst
├── projects.yaml.zst
└── manifest.json              (uploaded last - validates completeness)
```

**Manifest Schema:**
```json
{
  "snapshot_id": "20240615T040000Z",
  "created_at": "2024-06-15T04:00:00Z",
  "schema_version": "1.11.0",
  "fleet_db_key": "ex44/20240615T040000Z/fleet.db.zst",
  "attachments_manifest_key": "ex44/20240615T040000Z/attachments.manifest.json",
  "encryption": "none",
  "hoop_version": "0.1.0",
  "fleet_db_sha256": "abc123...",
  "fleet_db_size": 4096,
  "final_audit_hash": "deadbeef...",
  "config_backup": {
    "config_yml_hash": "...",
    "config_yml_size": 1234,
    "projects_yaml_hash": "...",
    "projects_yaml_size": 5678
  }
}
```

## Metrics Exposed

`hoop-daemon/src/metrics.rs` lines 764-771:
- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size of last successful backup
- `hoop_backup_failures_total` - Total failed backup runs
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

## API Endpoints

`hoop-daemon/src/api_backup.rs`:
- `POST /api/backup/trigger` - Manually trigger a backup run

## CLI Commands

`hoop-cli/src/backup.rs`:
- `hoop backup trigger [--addr]` - Trigger manual backup
- `hoop backup status [--addr]` - Show backup configuration and last run status

`hoop-cli/src/restore.rs`:
- `hoop restore --from s3://bucket/prefix/snapshot-id` - Restore from snapshot

## What Gets Backed Up

✅ `fleet.db` - SQLite database (audit log, Stitches, Patterns, Reflection Ledger)
✅ `attachments/` - Note audio, images, videos (incremental sync)
✅ `config.yml` - Non-secret configuration
✅ `projects.yaml` - Project registry

❌ Bead state - That's `br`'s job (in each workspace's `.beads/`)
❌ NEEDLE worker state - Separate system
❌ CLI session files - Each CLI owns these
❌ Git worktree state - Git's job

## Retry Logic

Exponential backoff with max 3 retries:
- Initial backoff: 2 seconds
- Max backoff: 60 seconds
- Formula: `backoff = min(backoff * 2, 60)`

## Security Considerations

1. Credentials **never** stored in config.yml
2. Credentials **never** written to logs (Debug impl redacts secrets)
3. Credentials from environment variables only
4. Age encryption optional (for untrusted S3 endpoints)
5. SHA-256 integrity verification on fleet.db
6. Audit hash chain verification on restore

## Testing

Comprehensive test coverage in all modules:
- `backup.rs` - Config parsing, credential validation, cron parsing
- `backup_pipeline.rs` - VACUUM INTO, zstd compression, retry logic
- `attachment_sync.rs` - Scan, diff, apply, tombstone pruning
- `snapshot_manifest.rs` - Serialization, version validation
- `config_backup.rs` - Hash computation, file reading
- `restore.rs` - S3 URI parsing, manifest validation, rollback logic

## Integration Points

1. **Daemon startup** (`lib.rs:2567-2597`):
   - Loads backup config
   - Resolves credentials
   - Starts scheduler when ready
   - Logs state (Ready/Disabled/NotConfigured)

2. **Shutdown coordination**:
   - Scheduler subscribes to shutdown channel
   - Gracefully exits on shutdown signal

3. **Metrics**:
   - Updates Prometheus metrics on success/failure
   - Exposed via `/metrics` endpoint

## Completion Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Backup runs on schedule | ✅ | `start_scheduler()` with cron parser |
| Credentials validated | ✅ | `BackupCredentials::from_env()` with warnings |
| Restore produces identical state | ✅ | Full restore with integrity checks |
| Documentation covers 4 DR scenarios | ✅ | `operations.md` sections 458-831 |
| age encryption with env var | ✅ | `age_encrypt()` uses `HOOP_BACKUP_AGE_KEY` |
| Metrics exposed | ✅ | 4 backup metrics in Prometheus format |
| API endpoint for manual trigger | ✅ | `POST /api/backup/trigger` |
| CLI commands | ✅ | `hoop backup trigger/status`, `hoop restore` |
| Comprehensive tests | ✅ | Unit tests in all modules |

**§15 Backups & disaster recovery is COMPLETE.**
