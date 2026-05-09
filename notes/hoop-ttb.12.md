# §15 Backups & Disaster Recovery — Implementation Verification

## Date: 2026-05-09

## Summary

The backup and disaster recovery implementation specified in plan §15 is **COMPLETE**. All components are implemented, tested, and documented.

## Closing Criteria Verification

### 1. Backup Configuration ✅
**Location:** `hoop-daemon/src/backup.rs`

**Implementation:**
- `BackupFileConfig` struct with all required fields:
  - `endpoint`: S3-compatible URL
  - `bucket`: S3 bucket name
  - `prefix`: Key prefix
  - `schedule`: Cron schedule (default: "0 4 * * *")
  - `retention_days`: Retention period (default: 30)
  - `encryption`: Age encryption flag
- `BackupCredentials` from environment variables:
  - `HOOP_BACKUP_ACCESS_KEY_ID`
  - `HOUP_BACKUP_SECRET_ACCESS_KEY`
  - `HOOP_BACKUP_AGE_KEY` (optional, for encryption)
- Config loading with validation (cron field count, URL format)
- Three-state configuration: `NotConfigured`, `Disabled`, `Ready`

### 2. Daily Snapshot Pipeline ✅
**Location:** `hoop-daemon/src/backup_pipeline.rs`

**Implementation:**
- `BackupPipeline::run_snapshot()` executes full pipeline:
  1. `VACUUM INTO` for fleet.db snapshot
  2. zstd compression (level 3)
  3. Optional age encryption
  4. S3 upload with retry (max 3 attempts, exponential backoff)
  5. Incremental attachment sync
  6. Config file backup
  7. Manifest upload (last, for completeness check)
- Cron scheduler: `start_scheduler()` checks every 60 seconds
- Manual trigger via API: `/api/backup/trigger`

### 3. Fleet.db Snapshot ✅
**Location:** `hoop-daemon/src/backup_pipeline.rs:474-509`

**Implementation:**
- `vacuum_into()`: SQLite `VACUUM INTO` to temp file
- Read-only connection to avoid blocking
- Produces self-contained snapshot

### 4. Attachments Incremental Sync ✅
**Location:** `hoop-daemon/src/attachment_sync.rs`

**Implementation:**
- `BackupManifest`: SHA-256 + size + mtime tracking
- `scan_attachments()`: Walk attachment directories
- `compute_diff()`: Added/changed/deleted detection
- `apply_diff()`: Update manifest with tombstones
- `prune_tombstones()`: Clean up old deletions
- Uploads only new/changed files

### 5. Config File Backup ✅
**Location:** `hoop-daemon/src/config_backup.rs`

**Implementation:**
- `ConfigBackup`: SHA-256 hashes for config.yml and projects.yaml
- `upload_config_to_snapshot()`: Compressed uploads
- Triggered on config reload and daily during backup

### 6. Manifest Upload ✅
**Location:** `hoop-daemon/src/snapshot_manifest.rs`

**Implementation:**
- `SnapshotManifest` struct with all required fields:
  - `snapshot_id`: ISO timestamp
  - `created_at`: ISO 8601
  - `schema_version`: From fleet::SCHEMA_VERSION
  - `fleet_db_key`: S3 key
  - `attachments_manifest_key`: Optional
  - `encryption`: "none" or "age"
  - `hoop_version`: Cargo version
  - `fleet_db_sha256`: Integrity hash
  - `fleet_db_size`: Compressed size
  - `final_audit_hash`: Audit chain hash
  - `config_backup`: Config metadata
- Uploaded **last** to signal completeness

### 7. Restore Command ✅
**Location:** `hoop-cli/src/restore.rs`

**Implementation:**
- `hoop restore --from s3://bucket/prefix/snapshot-id`
- Preconditions:
  - Daemon not running (checks control.sock and TCP port)
  - S3 credentials in environment
- Restore flow:
  1. Download and validate manifest
  2. Check schema version (rejects newer)
  3. Move existing ~/.hoop/ aside to ~/.hoop.rollback.YYYYMMDDTHHMMSSZ
  4. Download and restore fleet.db
  5. Restore attachments (if present)
  6. Restore config files
  7. Run schema migrations
  8. Verify audit hash chain
  9. Cleanup rollback dirs on success
  10. Automatic rollback on any failure

### 8. Encryption ✅
**Location:** `hoop-daemon/src/backup_pipeline.rs:535-562`

**Implementation:**
- `age_encrypt()`: Spawns `age --encrypt` subprocess
- Decrypt in restore: `decrypt_with_age()` uses `HOOP_BACKUP_AGE_IDENTITY`
- Graceful degradation: Encryption failure falls back to unencrypted upload with warning

### 9. S3-Compatible Upload ✅
**Location:** `hoop-daemon/src/backup_pipeline.rs:603-682`

**Implementation:**
- AWS SigV4 signing (manual implementation)
- Works with B2, AWS S3, MinIO, Garage
- Path-style URLs: `https://endpoint/bucket/key`

### 10. Metrics ✅
**Location:** `hoop-daemon/src/metrics.rs:765-878`

**Implementation:**
- `hoop_backup_last_success_timestamp`: Unix timestamp
- `hoop_backup_last_size_bytes`: Backup size
- `hoop_backup_failures_total`: Failure counter
- `hoop_backup_run_duration_seconds`: Histogram

### 11. Disaster Recovery Scenarios ✅
**Location:** `hoop-daemon/tests/disaster_recovery_runbook.rs`

**Tests cover all four scenarios:**
1. **Disk death** (`test_scenario_1_disk_death_restore_creates_fresh_state`)
   - Fresh host with no ~/.hoop/
   - Restore creates complete state
2. **fleet.db corruption** (`test_scenario_2_corruption_detected_by_integrity_check`)
   - Integrity check detection
   - Corrupted database preservation
3. **Accidental deletion** (`test_scenario_3_deletion_recovery_restores_from_backup`)
   - rm -rf ~/.hoop/ recovery
   - Rollback preservation
4. **Host migration** (`test_scenario_4_migration_preserves_projects_config`)
   - Config path updates
   - Project workspace handling

### 12. Documentation ✅
**Location:** `docs/operations.md:458-832`

**Complete documentation includes:**
- Configuration examples
- Environment variables
- What gets backed up
- Encryption setup
- Manual trigger
- All four disaster scenarios with step-by-step procedures
- Expected durations
- Pitfalls and error handling
- Rollback procedures

## Test Coverage

### Unit Tests
- `backup.rs`: Config parsing, credential loading, validation
- `attachment_sync.rs`: Diff computation, manifest management, tombstone pruning
- `snapshot_manifest.rs`: Version comparison, serialization
- `config_backup.rs`: Hash computation, file loading

### Integration Tests
- `disaster_recovery_runbook.rs`: Full scenario testing
- `restore.rs`: S3 URI parsing, rollback logic, version validation

## Success Criteria Met

✅ Backup runs on configurable schedule (cron)
✅ Credentials validated from environment variables
✅ S3-compatible endpoint (B2 default, any S3 API works)
✅ fleet.db daily via VACUUM INTO snapshot
✅ Attachments incremental sync (new/changed only)
✅ Config files on every change plus daily
✅ Each run writes manifest.json with schema version
✅ zstd compression (level 3)
✅ Optional age encryption
✅ Metrics: last_success_timestamp, last_size_bytes
✅ Restore: `hoop restore --from s3://bucket/prefix/snapshot-id`
✅ Restore is idempotent
✅ Restore precondition: `hoop serve` stopped
✅ Restore moves existing ~/.hoop/ aside for rollback
✅ Restore runs schema migrations
✅ All four DR scenarios tested and documented
✅ Age encryption with key in env var
✅ Documentation covers all DR scenarios

## Notes

- The implementation is complete and production-ready
- All code follows the patterns established in the plan
- Error handling includes proper rollback mechanisms
- Tests validate both happy path and failure scenarios
- Metrics provide observability for backup health
- Documentation is comprehensive and operator-focused

## Files Verified

- `hoop-daemon/src/backup.rs` — Configuration and credentials
- `hoop-daemon/src/backup_pipeline.rs` — Snapshot pipeline
- `hoop-daemon/src/attachment_sync.rs` — Incremental sync
- `hoop-daemon/src/config_backup.rs` — Config file backup
- `hoop-daemon/src/snapshot_manifest.rs` — Manifest schema
- `hoop-daemon/src/api_backup.rs` — REST API trigger
- `hoop-daemon/src/metrics.rs` — Backup metrics
- `hoop-cli/src/backup.rs` — CLI status/trigger
- `hoop-cli/src/restore.rs` — CLI restore command
- `hoop-daemon/tests/disaster_recovery_runbook.rs` — Integration tests
- `docs/operations.md` — Complete DR documentation
