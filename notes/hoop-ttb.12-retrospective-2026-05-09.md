# §15 Backups & disaster recovery - Retrospective (hoop-ttb.12)

## Bead ID: hoop-ttb.12
## Status: CLOSED ✅
## Date: 2026-05-09

## Implementation Summary

All components of the backup and disaster recovery system are implemented and tested:

**Core Components:**
- Config parsing and credential validation (`hoop-daemon/src/backup.rs`)
- Backup pipeline with VACUUM INTO, zstd compression, optional age encryption (`hoop-daemon/src/backup_pipeline.rs`)
- Incremental attachment sync with manifest-based diff engine (`hoop-daemon/src/attachment_sync.rs`)
- Snapshot manifest tying all pieces together (`hoop-daemon/src/snapshot_manifest.rs`)
- Config file backup for config.yml and projects.yaml (`hoop-daemon/src/config_backup.rs`)
- Restore CLI command with rollback safety (`hoop-cli/src/restore.rs`)
- REST API for manual backup trigger (`hoop-daemon/src/api_backup.rs`)

**Closing Criteria Met:**
1. ✅ Backup runs on configurable cron schedule with validated credentials
2. ✅ Restore produces identical state with SHA-256 integrity verification and audit hash chain validation
3. ✅ Documentation covers all four DR scenarios (disk death, fleet.db corruption, accidental deletion, host migration)
4. ✅ age encryption works with environment variable keys (HOUP_BACKUP_AGE_KEY for encryption, HOOP_BACKUP_AGE_IDENTITY or AGE_IDENTITY for decryption)

**Test Coverage:**
- 79 tests covering all backup and disaster recovery functionality
- Integration tests for all four DR scenarios
- Rollback mechanism tests
- Duration bound tests
- Pitfall detection tests

**Metrics Exported:**
- hoop_backup_last_success_timestamp
- hoop_backup_last_size_bytes
- hoop_backup_failures_total
- hoop_backup_run_duration_seconds

## Retrospective

### What worked

1. **VACUUM INTO snapshot approach** - Using SQLite's VACUUM INTO command produces a clean, consistent snapshot without blocking writes to the source database for the duration of the copy. This is ideal for daily backups of a long-running daemon.

2. **Incremental attachment sync with manifest-based diff** - The attachment sync engine efficiently handles large attachment directories by:
   - Computing SHA-256 hashes and mtimes for all files
   - Comparing against the prior manifest to detect additions, changes, and deletions
   - Only uploading new/changed files
   - Preserving deleted files as tombstones for configurable retention

3. **Automatic rollback on restore failure** - The restore command moves existing state aside before any destructive action and automatically rolls back if anything fails, providing a strong safety net for disaster recovery.

4. **Comprehensive documentation** - The four DR scenarios are thoroughly documented in `docs/operations.md` with step-by-step procedures, expected durations, common pitfalls, and verification steps.

### What didn't

1. **In-memory SQLite backup** - Initial attempts to use SQLite's backup API were insufficient because it still required holding locks during the entire backup operation. VACUUM INTO solved this by creating a completely independent snapshot file.

2. **Full attachment scans on every run** - Early attachment sync attempts that scanned all files on every run were too slow for large attachment directories. The manifest-based diff engine with SHA-256 hashes and mtimes solved this by only processing changes.

### Surprise

1. **Path validation complexity** - The complexity of handling both stitch attachments (`~/.hoop/attachments/`) and bead attachments (`<workspace>/.beads/attachments/`) required careful path validation and allowlist checking to prevent path traversal attacks. The solution uses ID validation for the prefix portion and canonicalize-and-check for the full path.

2. **Age encryption integration** - The age command-line tool integrates smoothly for encryption/decryption, but requires careful handling of:
   - Public key (HOOP_BACKUP_AGE_KEY) for encryption during backup
   - Private key file path (HOOP_BACKUP_AGE_IDENTITY or AGE_IDENTITY) for decryption during restore
   - Graceful degradation when encryption fails (uploads unencrypted with warning)

### Reusable patterns

1. **For backup systems requiring incremental sync:**
   - Use a manifest file with SHA-256 hashes and mtimes to detect changes
   - Only upload new/changed files
   - Preserve deleted files as tombstones for configurable retention
   - Upload the manifest last to validate completeness

2. **For restore safety:**
   - Always move existing state aside before any destructive action
   - Validate manifests before proceeding
   - Implement automatic rollback on failure
   - Clean up rollback directories on success

3. **For S3-compatible storage integration:**
   - Use AWS SigV4 signing for broad compatibility
   - Support custom endpoints (not just AWS S3)
   - Store credentials in environment variables only (never in config files)
   - Implement exponential backoff retry for transient failures

4. **For disaster recovery documentation:**
   - Cover all major failure scenarios with step-by-step procedures
   - Include expected durations to set proper expectations
   - Document common pitfalls and their solutions
   - Provide verification steps after each procedure

## Files Modified/Created

### Core Implementation
- `hoop-daemon/src/backup.rs` - Config parsing and credential validation
- `hoop-daemon/src/backup_pipeline.rs` - Backup pipeline with scheduler
- `hoop-daemon/src/attachment_sync.rs` - Incremental attachment sync
- `hoop-daemon/src/snapshot_manifest.rs` - Snapshot manifest schema
- `hoop-daemon/src/config_backup.rs` - Config file backup
- `hoop-cli/src/restore.rs` - Restore CLI command
- `hoop-daemon/src/api_backup.rs` - REST API for manual trigger

### Tests
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - Integration tests for DR scenarios
- Unit tests in each implementation file

### Documentation
- `docs/operations.md` - DR runbook for all four scenarios
- `notes/hoop-ttb.12-closing-summary-2026-05-09.md` - Closing summary
- `notes/hoop-ttb.12-retrospective-2026-05-09.md` - This retrospective

## Next Steps

The backup and disaster recovery system is production-ready. Future enhancements could include:
- Backup pruning to enforce retention_days at the S3 level
- Multi-region backup replication for additional safety
- Backup performance optimization for very large databases
- Continuous backup validation (periodic restore tests)
