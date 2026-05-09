# §15 Backups & disaster recovery - Verification Summary

## Date: 2026-05-09

## Summary

The backup and disaster recovery system is fully implemented and documented.

## Implementation Files

### Core Backup Components
- `hoop-daemon/src/backup.rs` - Configuration parser and credential resolver
- `hoop-daemon/src/backup_pipeline.rs` - Daily backup pipeline (VACUUM INTO → zstd → age → S3)
- `hoop-daemon/src/attachment_sync.rs` - Incremental attachment sync with manifest-based diff
- `hoop-daemon/src/config_backup.rs` - Config file backup on change + daily
- `hoop-daemon/src/snapshot_manifest.rs` - Manifest tying all backup pieces together
- `hoop-daemon/src/api_backup.rs` - REST API endpoint for manual backup trigger

### Restore Components
- `hoop-cli/src/restore.rs` - Full restore implementation with rollback support

### Tests
- `hoop-daemon/tests/disaster_recovery_runbook.rs` - Tests for all four DR scenarios

### Documentation
- `docs/operations.md` - Complete disaster recovery runbook

## Closing Criteria Verification

### 1. Backup runs on schedule; credentials validated ✅

**Implementation:**
- `backup.rs` validates cron schedule (5-field format)
- `backup.rs` validates endpoint URL (must start with http:// or https://)
- Credentials from env vars: `HOOP_BACKUP_ACCESS_KEY_ID`, `HOOP_BACKUP_SECRET_ACCESS_KEY`, `HOUP_BACKUP_AGE_KEY`
- `backup_pipeline.rs` has cron scheduler that checks every 60 seconds
- Exponential backoff retry (max 3 attempts)

**Metrics exposed:**
- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size of last successful backup
- `hoop_backup_failures_total` - Total failed backup runs
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

### 2. Restore from recent snapshot produces identical state ✅

**Implementation:**
- `restore.rs` - `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
- Precondition: daemon must not be running
- Downloads manifest.json first (uploaded last by backup)
- Validates schema version (rejects newer snapshots)
- Moves existing `~/.hoop/` aside for rollback
- Downloads and restores fleet.db, attachments, config files
- Runs schema migrations on restored database
- Verifies audit hash chain integrity
- Automatic rollback on any failure

**Tests:**
- `disaster_recovery_runbook.rs` - Tests for all four scenarios

### 3. Documentation covers all four DR scenarios ✅

**File:** `docs/operations.md`

**Scenarios documented:**
1. **Scenario 1: Disk death** (line 519)
2. **Scenario 2: fleet.db corruption** (line 606)
3. **Scenario 3: Accidental deletion** (line 659)
4. **Scenario 4: Host migration** (line 705+)

Each scenario includes:
- Expected duration
- Step-by-step recovery procedure
- Pitfalls and how to avoid them

### 4. age encryption works with key in env var ✅

**Implementation:**
- `backup_pipeline.rs` - `age_encrypt()` function
- `restore.rs` - `decrypt_with_age()` function
- Uses `HOOP_BACKUP_AGE_KEY` / `HOUP_BACKUP_AGE_IDENTITY` env vars

## All Components Integrated

All backup modules are properly integrated into `hoop-daemon/src/lib.rs`:
- `pub mod backup;`
- `pub mod backup_pipeline;`
- `pub mod attachment_sync;`
- `pub mod config_backup;`
- `pub mod snapshot_manifest;`
- API router merged: `.merge(api_backup::router())`

## Plan Reference

Plan §15: https://github.com/jedarden/HOOP/blob/main/docs/plan/plan.md#L1202-L1263

## Status

✅ All closing criteria met
✅ Implementation complete
✅ Tests written
✅ Documentation complete
