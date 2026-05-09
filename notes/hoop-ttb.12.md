# §15 Backups & disaster recovery - Completion Summary

## Task
Implement S3-compatible backup and disaster recovery for HOOP.

## What was implemented
All components were already in place. This task verified completeness.

### Core modules (hoop-daemon/src/)
1. **backup.rs** - Configuration parser and credential resolver
   - Reads `backup:` section from config.yml
   - Resolves S3 credentials from environment variables (never stored in YAML)
   - Validates cron schedule and endpoint URL format

2. **backup_pipeline.rs** - Daily backup pipeline
   - VACUUM INTO snapshot for fleet.db
   - zstd compression
   - Optional age encryption
   - S3 upload with AWS SigV4 signing
   - Exponential backoff retry (max 3 attempts)
   - Incremental attachment sync
   - Config file backup (config.yml, projects.yaml)
   - Manifest upload (last, for completeness validation)
   - Cron scheduler that fires daily per config schedule

3. **attachment_sync.rs** - Incremental attachment sync
   - Manifest-based diff engine
   - Tracks added/changed/deleted files
   - Tombstones for deleted files (configurable retention)
   - SHA-256 integrity verification
   - mtime+size shortcut for unchanged files

4. **config_backup.rs** - Config file backup
   - Backs up config.yml and projects.yaml
   - SHA-256 hash for integrity verification
   - Uploaded to each snapshot directory

5. **snapshot_manifest.rs** - Snapshot metadata
   - Ties all backup pieces together
   - Schema version validation (rejects newer snapshots)
   - SHA-256 integrity verification
   - Audit hash chain verification

6. **api_backup.rs** - REST API endpoint
   - POST /api/backup/trigger for manual backup

### CLI commands (hoop-cli/src/)
1. **backup.rs** - `hoop backup` commands
   - `hoop backup trigger` - Manual backup trigger
   - `hoop backup status` - Show configuration and last run status

2. **restore.rs** - `hoop restore` command
   - Fetches snapshot from S3
   - Validates manifest before destructive action
   - Moves existing ~/.hoop/ aside for rollback
   - Restores fleet.db, attachments, config
   - Runs schema migrations
   - Idempotent with automatic rollback on failure

### Tests (hoop-daemon/tests/)
1. **backup_restore_cycle.rs** - Integration tests
   - Backup/restore cycle produces identical state
   - Credentials validation (missing, invalid, valid)
   - age encryption with env key
   - Scheduler runs on cron schedule

2. **disaster_recovery_runbook.rs** - DR scenario tests
   - Scenario 1: Disk death
   - Scenario 2: fleet.db corruption
   - Scenario 3: Accidental deletion
   - Scenario 4: Host migration
   - Rollback mechanism
   - Pitfall detection

### Documentation (docs/operations.md)
Complete backup and disaster recovery runbook covering:
- Configuration (S3 endpoint, bucket, schedule, retention)
- What gets backed up (fleet.db, attachments, config files)
- Manual backup trigger
- Four DR scenarios with step-by-step procedures
- Rollback on failed restore
- Expected duration bounds
- Common pitfalls

## Closing criteria verification

1. ✅ **Backup runs on schedule; credentials validated**
   - `backup_pipeline::BackupPipeline::start_scheduler()` fires on cron schedule
   - `backup::BackupCredentials::from_env()` validates credentials
   - Missing credentials log warnings and disable backups

2. ✅ **Restore from recent snapshot produces identical state (verified)**
   - `backup_restore_cycle::backup_restore_cycle_produces_identical_state()` test
   - SHA-256 checksum verification for fleet.db
   - File size verification for attachments

3. ✅ **Documentation covers all four DR scenarios**
   - Disk death (Scenario 1)
   - fleet.db corruption (Scenario 2)
   - Accidental deletion (Scenario 3)
   - Host migration (Scenario 4)

4. ✅ **age encryption works with key in env var**
   - `backup_pipeline::BackupPipeline::age_encrypt()`
   - `restore::decrypt_with_age()`
   - `backup_restore_cycle::age_encryption_with_env_key()` test

## DR scenarios covered
- Disk death → restore to new host from S3 snapshot
- fleet.db corruption → restore from recent backup (≤1 day data loss)
- Accidental deletion → restore after rm -rf ~/.hoop/
- Host migration → fresh HOOP install + restore + workspace migration

## What's explicitly not backed up (as planned)
- Bead state (br's job, in each workspace's .beads/)
- NEEDLE worker state (separate)
- CLI session files (each CLI owns these)
- Git worktree state (git's job)

## References
- Plan §15
- Memory: project_armor_backup_strategy.md (ARMOR encrypted S3 proxy for B2)
