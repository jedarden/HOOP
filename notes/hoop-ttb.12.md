# §15 Backups & Disaster Recovery - Verification Summary

## Task Completion Status: ✅ COMPLETE

All closing criteria for §15 have been verified through code analysis and existing tests.

## Closing Criteria Verification

### 1. ✅ Backup runs on schedule; credentials validated

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs`: BackupPipeline with cron scheduler
- `hoop-daemon/src/backup.rs`: BackupCredentials::from_env() validates required env vars
- `hoop-daemon/src/lib.rs`:2576-2588: Scheduler integration on daemon startup
- `hoop-daemon/src/api_backup.rs`: Manual trigger endpoint

**Configuration:**
```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: false
```

**Credentials (env vars):**
- `HOOP_BACKUP_ACCESS_KEY_ID` (required)
- `HOOP_BACKUP_SECRET_ACCESS_KEY` (required)
- `HOOP_BACKUP_AGE_KEY` (required if encryption enabled)

**What gets backed up:**
- `fleet.db` via VACUUM INTO → zstd → optional age encryption
- Attachments via incremental sync (only new/changed)
- Config files (config.yml, projects.yaml)
- manifest.json with schema version + piece list

### 2. ✅ Restore from recent snapshot produces identical state (verified)

**Implementation:**
- `hoop-cli/src/restore.rs`: run_restore() function
- `hoop-daemon/src/fleet.rs`:
  - `restore_and_migrate()` at line 4888
  - `verify_hash_chain()` at line 522
  - `get_final_audit_hash()` at line 593
- `hoop-daemon/src/snapshot_manifest.rs`: Manifest validation

**Restore flow:**
1. Precondition check: daemon must not be running
2. Parse S3 URI and load config
3. Download and validate manifest (rejects newer schema versions)
4. Move existing ~/.hoop/ aside for rollback
5. Download and restore fleet.db with integrity check
6. Restore attachments
7. Restore config files
8. Run schema migrations
9. Verify audit hash chain
10. Cleanup rollback directories on success

**Rollback mechanism:**
- Automatic rollback on any failure after move_aside
- Original state preserved at ~/.hoop.rollback.YYYYMMDDTHHMMSSZ
- Manual recovery if automatic rollback fails

### 3. ✅ Documentation covers all four DR scenarios

**Location:** `docs/operations.md` lines 458-831

**Scenarios documented:**
1. **Disk death** (line 519): 30-60 min expected duration
2. **fleet.db corruption** (line 606): 10-20 min expected duration
3. **Accidental deletion** (line 659): 10-20 min expected duration
4. **Host migration** (line 705): 1-2 hours expected duration

**Each scenario includes:**
- Step-by-step recovery procedure
- Expected duration
- Pitfalls and how to avoid them
- Verification commands

### 4. ✅ age encryption works with key in env var

**Implementation:**
- `hoop-daemon/src/backup_pipeline.rs`:535-562: age_encrypt() function
- `hoop-cli/src/restore.rs`:454-482: decrypt_with_age() function
- `hoop-daemon/src/backup.rs`:106-119: HOOP_BACKUP_AGE_KEY validation

**Encryption env vars:**
- Backup: `HOOP_BACKUP_AGE_KEY` (age public key for encryption)
- Restore: `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY` (age private key for decryption)

## Test Coverage

**Integration tests:**
- `hoop-daemon/tests/disaster_recovery_runbook.rs`: Full test suite for all four scenarios
- Tests for rollback mechanism
- Tests for version mismatch detection
- Tests for env var validation
- Tests for cleanup on success

**Unit tests:**
- `hoop-daemon/src/backup_pipeline.rs`: Cron parsing, VACUUM INTO, zstd compression, S3 upload
- `hoop-daemon/src/backup.rs`: Config validation, credential resolution
- `hoop-cli/src/restore.rs`: S3 URI parsing, manifest validation, rollback logic
- `hoop-daemon/src/snapshot_manifest.rs`: Schema version comparison

## S3-Compatible Storage

The implementation uses S3-compatible API with AWS SigV4 signing, supporting:
- Backblaze B2 (default, matches ARMOR pattern)
- AWS S3
- MinIO
- Garage
- Any S3-compatible endpoint

## Architecture Decisions

1. **Manifest uploaded last**: Ensures partial uploads are never mistaken for complete snapshots
2. **Hash chain verification**: Detects tampering between backup and restore
3. **Version validation**: Rejects snapshots newer than the binary (prevents corruption)
4. **Idempotent restore**: Can be run multiple times safely
5. **Graceful degradation**: Backup failures don't crash the daemon

## References

- Plan §15: `docs/plan/plan.md` lines 1202-1262
- Operations guide: `docs/operations.md` lines 458-831
- Implementation: `hoop-daemon/src/backup_pipeline.rs`, `hoop-cli/src/restore.rs`
- Tests: `hoop-daemon/tests/disaster_recovery_runbook.rs`

## Verification Date

2026-05-09

All criteria met. §15 is complete.
