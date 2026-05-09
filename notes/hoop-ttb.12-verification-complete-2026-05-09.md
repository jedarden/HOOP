# §15 Backups & disaster recovery - Final Verification Complete

**Date:** 2026-05-09
**Bead:** hoop-ttb.12

## Summary

All §15 Backups & disaster recovery closing criteria verified as complete.

## Implementation Verified

### Core Components (all present in codebase)
- `hoop-daemon/src/backup.rs` - Config parser and credential resolver
- `hoop-daemon/src/backup_pipeline.rs` - Daily backup pipeline with cron scheduler
- `hoop-daemon/src/attachment_sync.rs` - Incremental attachment sync
- `hoop-daemon/src/snapshot_manifest.rs` - Manifest with schema version pinning
- `hoop-daemon/src/config_backup.rs` - Config file backup
- `hoop-cli/src/restore.rs` - CLI restore command with rollback
- `hoop-cli/src/backup.rs` - CLI backup trigger/status commands
- `hoop-daemon/src/api_backup.rs` - REST API endpoint

### Closing Criteria Verification

**1. Backup runs on schedule; credentials validated** ✅
- Config from `~/.hoop/config.yml` `backup:` section
- Credentials from env vars: `HOOP_BACKUP_ACCESS_KEY_ID`, `HOOP_BACKUP_SECRET_ACCESS_KEY`, `HOOP_BACKUP_AGE_KEY`
- Cron scheduler checks every 60s with at-most-once-per-day guard
- Exponential backoff retry (max 3 attempts)

**2. Restore from recent snapshot produces identical state** ✅
- `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
- Precondition: daemon not running
- Manifest validation before destructive action
- SHA-256 integrity verification
- Audit hash chain verification
- Automatic rollback on failure
- Schema migrations on restored database

**3. Documentation covers all four DR scenarios** ✅
- `docs/operations.md` (2121 lines) - Complete DR runbook
- Scenario 1: Disk death (line 519)
- Scenario 2: fleet.db corruption (line 606)
- Scenario 3: Accidental deletion (line 659)
- Scenario 4: Host migration (line 705)

**4. age encryption works with key in env var** ✅
- `age_encrypt()` in backup_pipeline.rs
- `decrypt_with_age()` in restore.rs
- Uses `HOOP_BACKUP_AGE_KEY` / `HOOP_BACKUP_AGE_IDENTITY` env vars

## Integration Tests

`hoop-daemon/tests/disaster_recovery_runbook.rs` - Tests for all four scenarios:
- Disk death restore
- Corruption detection and recovery
- Deletion recovery
- Host migration
- Rollback mechanism
- Duration bounds
- Pitfall detection

## Metrics Exported

- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size of last successful backup
- `hoop_backup_failures_total` - Total failed backup runs
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

## Child Beads Status

All child beads closed:
- hoop-ttb.12.1 - Backup config schema + env-var credential resolver ✅
- hoop-ttb.12.2 - Daily fleet.db snapshot pipeline ✅
- hoop-ttb.12.3 - Incremental attachments sync ✅
- hoop-ttb.12.4 - Backup manifest.json format + schema version pinning ✅
- hoop-ttb.12.5 - docs/operations.md DR runbook ✅
- hoop-ttb.12.6 - hoop restore: newer-than-current rejection + rollback ✅

## Conclusion

The backup and disaster recovery system (§15) is fully implemented, tested, and documented. All closing criteria are met.
