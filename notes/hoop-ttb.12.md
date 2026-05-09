# §15 Backups & Disaster Recovery — Implementation Summary

## Status: ✅ COMPLETE

All closing criteria for §15 have been verified as implemented.

## Closing Criteria Verification

### 1. Backup runs on schedule; credentials validated ✅
- **Location:** `hoop-daemon/src/backup_pipeline.rs` (scheduler), `hoop-daemon/src/backup.rs` (credential resolution)
- **Implementation:**
  - Cron-based scheduler with 5-field expression matching
  - S3 credentials resolved from `HOOP_BACKUP_ACCESS_KEY_ID` and `HOOP_BACKUP_SECRET_ACCESS_KEY`
  - Optional age encryption key from `HOOP_BACKUP_AGE_KEY`
  - Scheduler started in `lib.rs` when backup config is `Ready`
- **Verification:** Configuration validation, credential checking, and scheduler startup all implemented

### 2. Restore from recent snapshot produces identical state ✅
- **Location:** `hoop-cli/src/restore.rs`, `hoop-daemon/tests/backup_restore_cycle.rs`
- **Implementation:**
  - `hoop restore --from s3://<bucket>/<prefix>/<snapshot-id>` CLI command
  - Manifest validation before any destructive action
  - Rollback mechanism: `~/.hoop.rollback.YYYYMMDDTHHMMSSZ`
  - Integrity checks with SHA-256 verification
  - Schema migrations on restored database
- **Verification:** Integration test `backup_restore_cycle_produces_identical_state` verifies checksums match

### 3. Documentation covers all four DR scenarios ✅
- **Location:** `docs/operations.md` (lines 516-832)
- **Coverage:**
  - **Scenario 1: Disk death** — Full recovery procedure with host provisioning steps
  - **Scenario 2: fleet.db corruption** — Corruption detection and restore steps
  - **Scenario 3: Accidental deletion** — Recovery from `rm -rf ~/.hoop/`
  - **Scenario 4: Host migration** — Complete migration checklist
  - **Rollback on failed restore** — Automatic rollback mechanism documented

### 4. age encryption works with key in env var ✅
- **Location:** `hoop-daemon/src/backup_pipeline.rs` (age_encrypt function)
- **Implementation:**
  - Optional encryption via `encryption: true` in config.yml
  - Reads `HOOP_BACKUP_AGE_KEY` for public key
  - Decrypt during restore with `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY`
- **Verification:** Test `age_encryption_with_env_key` in `backup_restore_cycle.rs`

## Implementation Architecture

### Backup Pipeline Components
| Component | File | Purpose |
|-----------|------|---------|
| Config loader | `backup.rs` | Parse `backup:` section, resolve env vars |
| Snapshot pipeline | `backup_pipeline.rs` | VACUUM INTO → zstd → age → S3 |
| Attachment sync | `attachment_sync.rs` | Incremental attachment backup |
| Config backup | `config_backup.rs` | Backup config.yml and projects.yaml |
| Manifest | `snapshot_manifest.rs` | Tie pieces together, schema version |

### CLI Commands
| Command | Location | Purpose |
|---------|----------|---------|
| `hoop backup trigger` | `hoop-cli/src/backup.rs` | Manual backup trigger |
| `hoop backup status` | `hoop-cli/src/backup.rs` | Show backup configuration and status |
| `hoop restore --from s3://...` | `hoop-cli/src/restore.rs` | Restore from snapshot |

### REST API
| Endpoint | Location | Purpose |
|----------|----------|---------|
| `POST /api/backup/trigger` | `api_backup.rs` | Manual backup trigger via API |

### Metrics (§16.6)
| Metric | Type | Purpose |
|--------|------|---------|
| `hoop_backup_last_success_timestamp` | Gauge | Unix timestamp of last successful backup |
| `hoop_backup_last_size_bytes` | Gauge | Size of last successful backup |
| `hoop_backup_failures_total` | Counter | Total failed backup runs |
| `hoop_backup_run_duration_seconds` | Histogram | Backup run duration |

## Configuration Example

```yaml
# ~/.hoop/config.yml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false              # set to true for age encryption
```

Credentials (environment variables only):
```bash
export HOOP_BACKUP_ACCESS_KEY_ID="your-access-key"
export HOOP_BACKUP_SECRET_ACCESS_KEY="your-secret-key"
# If encryption is enabled:
export HOOP_BACKUP_AGE_KEY="age1...your-public-key"
```

## What Gets Backed Up
- `fleet.db` — SQLite `VACUUM INTO` snapshot (daily)
- Attachments — Incremental sync (only new/changed files)
- Config files — `config.yml` and `projects.yaml` (on change + daily)
- `manifest.json` — Snapshot metadata with schema version (uploaded last)

## What's NOT Backed Up (per §15.6)
- Bead state (`br`'s job, in each workspace's `.beads/`)
- NEEDLE worker state (separate)
- CLI session files (each CLI owns these)
- Git worktree state (git's job)

## Test Coverage
- `backup_restore_cycle.rs` — Backup-restore cycle, credentials validation, age encryption
- `disaster_recovery_runbook.rs` — All four DR scenarios
- Unit tests in each module (cron parsing, manifest validation, etc.)

## References
- Plan §15: `/home/coding/HOOP/docs/plan/plan.md` (lines 1202-1263)
- Operations guide: `/home/coding/HOOP/docs/operations.md` (lines 458-832)
