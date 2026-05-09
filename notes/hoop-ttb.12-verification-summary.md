# §15 Backups & disaster recovery - Final Verification

## Date: 2026-05-09

## Summary

The backup and disaster recovery system (§15) is fully implemented, tested, and documented.

## Implementation Overview

### Core Components

1. **Backup Pipeline** (`hoop-daemon/src/backup_pipeline.rs`)
   - Daily cron-scheduled backups (configurable, default 04:00)
   - SQLite `VACUUM INTO` for consistent snapshots
   - zstd compression (level 3)
   - Optional age encryption (via `HOOP_BACKUP_AGE_KEY`)
   - S3-compatible storage upload with exponential backoff retry (max 3 attempts)
   - Incremental attachment sync (only new/changed files)
   - Config file backup (config.yml, projects.yaml)
   - Manifest generation (uploaded last for completeness validation)

2. **Configuration** (`hoop-daemon/src/backup.rs`)
   - Reads `backup:` section from `~/.hoop/config.yml`
   - Credentials from env vars only (never in config):
     - `HOOP_BACKUP_ACCESS_KEY_ID`
     - `HOOP_BACKUP_SECRET_ACCESS_KEY`
     - `HOOP_BACKUP_AGE_KEY` (optional, for encryption)
   - Validates cron schedule format
   - Validates endpoint URL
   - Three states: NotConfigured, Disabled, Ready

3. **Restore** (`hoop-cli/src/restore.rs`)
   - Command: `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>`
   - Precondition: daemon must not be running
   - Downloads and validates manifest first
   - Schema version validation (rejects newer snapshots)
   - Atomic rollback on failure
   - Restores: fleet.db, attachments, config files
   - Runs schema migrations
   - Verifies audit hash chain

4. **Attachment Sync** (`hoop-daemon/src/attachment_sync.rs`)
   - Manifest-based diff engine
   - Tracks files by SHA-256, size, mtime
   - Tombstones for deleted files (configurable retention)
   - Prefixes: `stitch/` for ~/.hoop/attachments, `bead/` for .beads/attachments

5. **Config Backup** (`hoop-daemon/src/config_backup.rs`)
   - Backs up config.yml and projects.yaml
   - SHA-256 hash for integrity verification
   - Uploaded with each snapshot

6. **Snapshot Manifest** (`hoop-daemon/src/snapshot_manifest.rs`)
   - Ties all backup pieces together
   - Includes schema version for validation
   - Uploaded last (ensures incomplete snapshots are rejected)

### CLI Interface

- `hoop backup trigger` - Manually trigger a backup
- `hoop backup status` - Show backup configuration and last run status
- `hoop restore --from s3://...` - Restore from a snapshot

### API Endpoint

- `POST /api/backup/trigger` - Manual backup trigger via REST API

### Metrics

- `hoop_backup_last_success_timestamp` - Unix timestamp of last successful backup
- `hoop_backup_last_size_bytes` - Size of last successful backup
- `hoop_backup_failures_total` - Total failed backup runs
- `hoop_backup_run_duration_seconds` - Wall-clock duration histogram

### Documentation

`docs/operations.md` includes:
- Configuration instructions
- All four DR scenarios with step-by-step procedures:
  1. Disk death
  2. fleet.db corruption
  3. Accidental deletion
  4. Host migration
- Pitfalls and solutions for each scenario
- Rollback procedures

### Tests

`hoop-daemon/tests/disaster_recovery_runbook.rs`:
- Tests for all four DR scenarios
- Rollback mechanism tests
- Pitfall detection tests
- Manifest validation tests
- Duration bound verification

## Closing Criteria Status

- ✅ Backup runs on schedule; credentials validated
- ✅ Restore from recent snapshot produces identical state (verified)
- ✅ Documentation covers all four DR scenarios
- ✅ age encryption works with key in env var

## DR Scenarios Covered

1. **Disk death** - Restore to new host from S3 snapshot (30-60 min)
2. **fleet.db corruption** - Restore from backup, lose at most one day (10-20 min)
3. **Accidental deletion** - Restore after `rm -rf ~/.hoop/` (10-20 min)
4. **Host migration** - Fresh HOOP install + restore (1-2 hours)

## What Gets Backed Up

- `fleet.db` - SQLite database (audit log, Stitches, Patterns, Reflection Ledger)
- `attachments/` - Note audio, image/video uploads, screen-capture recordings
- `config.yml` - Non-secret configuration
- `projects.yaml` - Project registry

## What's Explicitly Not Backed Up

- Bead state (that's `br`'s job, in each workspace's `.beads/`)
- NEEDLE worker state (separate)
- CLI session files (each CLI owns these)
- Git worktree state (git's job)

## S3 Storage Layout

```
s3://<bucket>/<prefix>/<snapshot-id>/
  ├── manifest.json              # Uploaded last, validates completeness
  ├── fleet.db.zst[.age]        # Compressed database, optionally encrypted
  ├── config.yml.zst             # Compressed config
  ├── projects.yaml.zst          # Compressed projects registry
  ├── attachments.manifest.json  # Attachment inventory
  └── attachments/               # New/changed attachments (incremental)
      └── <rel-path>.zst
```

## Encryption

When `encryption: true` in config.yml:
- `fleet.db.zst.age` - Age-encrypted database
- Config files are also encrypted
- Attachments are NOT encrypted (large files, less sensitive)
- Decryption requires `HOOP_BACKUP_AGE_IDENTITY` env var

## Retention

- Configurable `retention_days` (default: 30)
- Tombstones for deleted attachments are pruned after retention period
- Old snapshots can be manually deleted from S3

## Integration Points

- Backup scheduler started in `hoop-daemon/src/lib.rs` when daemon starts
- Metrics exported via `/metrics` endpoint
- API endpoint integrated into main router
- CLI commands integrated into main CLI
