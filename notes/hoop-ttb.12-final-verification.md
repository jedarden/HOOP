# §15 Backups & disaster recovery - Final Verification

## Date: 2026-05-09

## Summary

Verified that the backup and disaster recovery system (§15) is fully implemented and all closing criteria are met.

## Implementation verification

### 1. Backup runs on schedule; credentials validated ✅

**Files:**
- `hoop-daemon/src/backup.rs` - Configuration parser and credential resolver
  - Reads `backup:` section from `~/.hoop/config.yml`
  - Credentials from env vars: `HOOP_BACKUP_ACCESS_KEY_ID`, `HOOP_BACKUP_SECRET_ACCESS_KEY`, `HOOP_BACKUP_AGE_KEY`
  - Validates cron schedule (5-field format)
  - Validates endpoint URL (must start with http:// or https://)

- `hoop-daemon/src/backup_pipeline.rs` - Daily backup pipeline
  - Cron scheduler checks every 60 seconds
  - `VACUUM INTO` → zstd compression → optional age encryption → S3 upload
  - Exponential backoff retry (max 3 attempts)

**Metrics exposed:**
- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size of last successful backup
- `hoop_backup_failures_total` - Total failed backup runs
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

### 2. Restore from recent snapshot produces identical state ✅

**Files:**
- `hoop-cli/src/restore.rs` - Full restore implementation
  - `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
  - Precondition: daemon must not be running
  - Downloads manifest.json first (uploaded last by backup)
  - Validates schema version (rejects newer snapshots)
  - Moves existing `~/.hoop/` aside for rollback
  - Downloads and restores fleet.db, attachments, config files
  - Runs schema migrations on restored database
  - Verifies audit hash chain integrity
  - Automatic rollback on any failure

**Tests:**
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - Tests for all four scenarios

### 3. Documentation covers all four DR scenarios ✅

**File:** `docs/operations.md` (lines 458-831)

**Scenarios documented:**
1. **Scenario 1: Disk death** (line 519) - 30-60 minutes expected duration
2. **Scenario 2: fleet.db corruption** (line 606) - 10-20 minutes expected duration
3. **Scenario 3: Accidental deletion** (line 659) - 10-20 minutes expected duration
4. **Scenario 4: Host migration** (line 705) - 1-2 hours expected duration

### 4. age encryption works with key in env var ✅

**Implementation:**
- `backup_pipeline.rs` - `age_encrypt()` function (lines 535-562)
- `restore.rs` - `decrypt_with_age()` function (lines 454-482)
- Uses `HOOP_BACKUP_AGE_KEY` / `HOOP_BACKUP_AGE_IDENTITY` env vars

## Closing criteria met

- ✅ Backup runs on schedule; credentials validated
- ✅ Restore from recent snapshot produces identical state (verified)
- ✅ Documentation covers all four DR scenarios
- ✅ age encryption works with key in env var

## Additional components verified

- `hoop-daemon/src/snapshot_manifest.rs` - Manifest structure with validation
- `hoop-daemon/src/config_backup.rs` - Config file backup (config.yml, projects.yaml)
- `hoop-daemon/src/api_backup.rs` - REST API endpoint for manual backup trigger
- `hoop-daemon/src/attachment_sync.rs` - Incremental attachment sync

## Test coverage

All four disaster scenarios have dedicated tests in `disaster_recovery_runbook.rs`:
- `test_scenario_1_disk_death_restore_creates_fresh_state`
- `test_scenario_2_corruption_detected_by_integrity_check`
- `test_scenario_3_deletion_recovery_restores_from_backup`
- `test_scenario_4_migration_preserves_projects_config`

Plus rollback and pitfall detection tests.
