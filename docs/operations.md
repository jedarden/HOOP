# HOOP Operations Guide

This guide covers the operational aspects of running HOOP in production, including systemd service management, upgrades, backups, and disaster recovery procedures.

## Systemd user service

HOOP includes a systemd user service that runs the daemon as your user. This provides:

- Automatic startup on login
- Automatic restart on failure with rate limiting
- Journal integration for log viewing

### Installing the service

```bash
hoop install-systemd
```

This writes `~/.config/systemd/user/hoop.service` and prints instructions to enable and start the service.

### Service unit file

The installed service file includes:

| Directive | Value | Purpose |
|-----------|-------|---------|
| `Type` | `simple` | Daemon forks to background |
| `Restart` | `on-failure` | Restart on non-zero exit |
| `RestartSec` | `5s` | Wait 5 seconds between restarts |
| `StartLimitBurst` | `5` | Max 5 restarts |
| `StartLimitIntervalSec` | `5min` | Within 5 minute window |
| `TimeoutStartSec` | `30s` | Give daemon 30s to start |
| `TimeoutStopSec` | `30s` | Give daemon 30s to stop gracefully |

### Enabling and starting

```bash
# Reload systemd to pick up the new unit file
systemctl --user daemon-reload

# Enable the service to start on login
systemctl --user enable hoop

# Start the service now
systemctl --user start hoop
```

### Managing the service

```bash
# Check service status
systemctl --user status hoop

# Stop the service
systemctl --user stop hoop

# Restart the service
systemctl --user restart hoop

# Disable (don't start on login)
systemctl --user disable hoop
```

### Viewing logs

```bash
# Follow logs in real-time
journalctl --user -u hoop -f

# View last 100 lines
journalctl --user -u hoop -n 100

# View logs since today
journalctl --user -u hoop --since today

# View logs with priority level errors and above
journalctl --user -u hoop -p err
```

### Service lifecycle on failure

When the daemon crashes:

1. systemd waits `RestartSec` (5s)
2. Restarts the daemon
3. Increments the restart counter
4. If `StartLimitBurst` (5) is reached within `StartLimitIntervalSec` (5min), the daemon is not restarted and enters a failed state

To reset the failure state:

```bash
systemctl --user reset-failed hoop
systemctl --user start hoop
```

### Troubleshooting

| Symptom | Check |
|---------|-------|
| Service won't start | `journalctl --user -u hoop -n 50` |
| Service repeatedly crashes | Check logs for panic or assertion failure |
| Port 3000 already in use | `lsof -i :3000` or change port in unit file |
| Service not starting on login | `systemctl --user status hoop` - is it enabled? |

## Upgrading

```bash
# 1. Pull the new binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Restart the service
systemctl --user restart hoop
```

State in `~/.hoop/` persists across upgrades. Schema migrations run on startup.

## Backups

HOOP includes automated daily backups to S3-compatible storage (Backblaze B2, AWS S3, MinIO, Garage, etc.). Backups are configured in `~/.hoop/config.yml`:

```yaml
backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-<operator>
  prefix: ex44/
  schedule: "0 4 * * *"         # daily 04:00 local
  retention_days: 30
  encryption: false              # set to true for age encryption
```

Credentials are set via environment variables (never in config files):

```bash
export HOOP_BACKUP_ACCESS_KEY_ID="your-access-key"
export HOOP_BACKUP_SECRET_ACCESS_KEY="your-secret-key"
# If encryption is enabled:
export HOOP_BACKUP_AGE_KEY="age1...your-public-key"
```

### What gets backed up

- **`fleet.db`** — SQLite database containing audit log, Stitches, Patterns, Reflection Ledger
- **Attachments** — Note audio, image/video uploads, screen-capture recordings (incremental sync)
- **Config files** — `config.yml`, `projects.yaml`

Each backup produces a **snapshot** with a unique ID (ISO 8601 timestamp, e.g., `20240615T040000Z`) and uploads:

1. `fleet.db.zst` — Compressed database snapshot
2. `attachments.manifest.json` — Attachment inventory
3. `attachments/*.zst` — New or changed attachments (incremental)
4. `manifest.json` — Snapshot metadata (uploaded last, validates completeness)

### Encryption (optional)

When `encryption: true` is set in config:

- `fleet.db.zst.age` — Age-encrypted database
- Attachments are NOT encrypted (large files, less sensitive)

To decrypt during restore, set `HOOP_BACKUP_AGE_IDENTITY` or `AGE_IDENTITY` to your age private key path.

### Manual backup

To trigger an immediate backup outside the scheduled time:

```bash
# The daemon must be running
systemctl --user is-active hoop

# Trigger via the API (requires authentication)
curl -X POST http://localhost:3000/api/backup/trigger
```

## Disaster Recovery

This section covers four disaster scenarios with step-by-step recovery procedures.

### Scenario 1: Disk death

**Situation:** The host's disk fails completely. HOOP data is gone, but backups exist in S3.

**Expected duration:** 30-60 minutes (provisioning new host + downloading backup)

**Recovery procedure:**

1. **Provision a new host** (same Hetzner EX44 class or equivalent)
   - Install OS dependencies: Rust toolchain, tmux, git 2.5+
   - Join Tailscale network
   - Install `br` (beads_rust) at `~/.local/bin/br`

2. **Install HOOP binary**

```bash
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop
```

3. **Set up S3 credentials** (same as original host)

```bash
export HOOP_BACKUP_ENDPOINT="https://s3.us-west-000.backblazeb2.com"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
# If backups were encrypted:
export HOOP_BACKUP_AGE_IDENTITY="~/.age-key.txt"
```

4. **List available snapshots** (using S3 CLI or your provider's web UI)

```bash
# Using AWS CLI (configured for B2):
aws --endpoint-url=https://s3.us-west-000.backblazeb2.com \
  s3 ls s3://hoop-backups-<operator>/ex44/ | tail -10
```

5. **Stop HOOP** (if systemd service was already started)

```bash
systemctl --user stop hoop || true
```

6. **Restore from the latest snapshot**

```bash
hoop restore --from s3://hoop-backups-<operator>/ex44/20240615T040000Z
```

This will:
- Download and validate the manifest
- Move any existing `~/.hoop/` aside to `~/.hoop.rollback.YYYYMMDDTHHMMSSZ`
- Restore `fleet.db`, attachments, and config
- Run schema migrations to bring the database up to current HOOP version
- Clean up rollback directories on success

7. **Verify restore**

```bash
# Check database integrity
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"

# Verify projects are registered
hoop projects list

# Start the daemon
systemctl --user start hoop

# Check logs
journalctl --user -u hoop -n 50
```

8. **Reinstall systemd service** (if needed)

```bash
hoop install-systemd
systemctl --user daemon-reload
systemctl --user enable hoop
```

**Pitfalls:**
- **Version mismatch:** If the snapshot's schema version is newer than the installed HOOP binary, restore will fail with a clear error. Upgrade HOOP before restoring.
- **Missing credentials:** S3 credentials must be set as environment variables. If not set, restore will fail with "Set HOOP_BACKUP_ENDPOINT..."
- **Encryption key:** If backups were encrypted and `HOUP_BACKUP_AGE_IDENTITY` is not set, restore will fail during fleet.db decryption.
- **NEEDLE workspaces:** HOOP restores its own state, but NOT bead state in each project's `.beads/`. If those were on the failed disk, you'll need to restore those separately from their own backups.

### Scenario 2: fleet.db corruption

**Situation:** `~/.hoop/fleet.db` is corrupted (disk error, crash during write, etc.). The daemon won't start.

**Expected duration:** 10-20 minutes

**Recovery procedure:**

1. **Confirm corruption**

```bash
# Stop the daemon if running
systemctl --user stop hoop

# Try to open the database
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"
# Expected output: "ok"
# If corrupted: "database disk image is malformed" or similar
```

2. **Preserve the corrupted database for analysis**

```bash
cp ~/.hoop/fleet.db ~/.hoop/fleet.db.corrupted.$(date +%Y%m%d%H%M)
```

3. **List available snapshots**

```bash
# Using AWS CLI:
aws --endpoint-url=$HOOP_BACKUP_ENDPOINT \
  s3 ls s3://hoop-backups-<operator>/ex44/ | tail -5
```

4. **Set restore credentials**

```bash
export HOOP_BACKUP_ENDPOINT="https://s3.us-west-000.backblazeb2.com"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
```

5. **Restore from the most recent snapshot**

```bash
hoop restore --from s3://hoop-backups-<operator>/ex44/<snapshot-id>
```

**Pitfalls:**
- **Data loss:** You'll lose all changes since the last backup (up to 24 hours if daily backups). This includes Stitches, audit log entries, and Reflection Ledger changes.
- **Attachment desync:** If attachments were added after the last backup, they'll be missing from the restored state but still exist on disk. The attachment manifest will be inconsistent.
- **NEEDLE state:** Bead state in `.beads/` directories is unaffected by fleet.db corruption. Workers continue running; only HOOP's view is lost.

### Scenario 3: Accidental deletion

**Situation:** Operator accidentally ran `rm -rf ~/.hoop/` or deleted critical files.

**Expected duration:** 10-20 minutes

**Recovery procedure:**

1. **Stop any running daemon** (to prevent further writes to a now-missing state)

```bash
systemctl --user stop hoop || true
```

2. **Set restore credentials**

```bash
export HOOP_BACKUP_ENDPOINT="https://s3.us-west-000.backblazeb2.com"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
```

3. **Restore from the most recent snapshot**

```bash
hoop restore --from s3://hoop-backups-<operator>/ex44/<snapshot-id>
```

4. **Verify restoration**

```bash
# Check that projects.yaml exists
cat ~/.hoop/projects.yaml

# Check database integrity
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"

# Start the daemon
systemctl --user start hoop
journalctl --user -u hoop -n 50
```

**Pitfalls:**
- **Same as Scenario 2:** Data loss since last backup, attachment desync.
- **If `projects.yaml` was not backed up:** The restore command preserves `projects.yaml` from the old (now-deleted) state if it still exists in the rollback directory. If completely gone, you'll need to re-register projects with `hoop projects add`.

### Scenario 4: Host migration

**Situation:** Migrating HOOP from one host to another (e.g., upgrading hardware, changing datacenter).

**Expected duration:** 1-2 hours (depends on data transfer)

**Recovery procedure:**

1. **On the OLD host: Final backup**

```bash
# Ensure the daemon is running and backups are configured
systemctl --user is-active hoop

# Trigger a final backup before shutdown
curl -X POST http://localhost:3000/api/backup/trigger

# Wait for backup to complete (check logs)
journalctl --user -u hoop -f | grep "Backup.*completed"

# Note the snapshot ID from the logs
```

2. **On the OLD host: Stop HOOP and NEEDLE workers**

```bash
# Stop HOOP daemon
systemctl --user stop hoop

# Stop NEEDLE workers (if managed separately)
# This depends on your NEEDLE setup
```

3. **On the NEW host: Prepare environment**

```bash
# Install OS dependencies
sudo apt-get install -y build-essential tmux git

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install br (beads_rust)
cargo install --git https://github.com/dicklesworthstone/beads_rust br

# Join Tailscale network (follow your org's process)
```

4. **On the NEW host: Install HOOP**

```bash
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# Install systemd service
hoop install-systemd
systemctl --user daemon-reload
systemctl --user enable hoop
```

5. **On the NEW host: Restore project workspaces**

HOOP's backup does NOT include project code or `.beads/` state. You need to migrate those separately:

```bash
# Option A: Git clone (for projects in git)
cd ~/
git clone <your-repo-url> project-name

# Option B: rsync from old host (if still accessible)
rsync -avz old-host:/home/coding/project-name ./

# Option C: Restore from your own project backups
```

6. **On the NEW host: Set restore credentials and restore**

```bash
export HOOP_BACKUP_ENDPOINT="https://s3.us-west-000.backblazeb2.com"
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"

hoop restore --from s3://hoop-backups-<operator>/ex44/<snapshot-id>
```

7. **On the NEW host: Update config if needed**

```bash
# If Tailscale IP changed, update config.yml
# If paths to projects changed, update projects.yaml
vim ~/.hoop/projects.yaml
```

8. **On the NEW host: Start HOOP**

```bash
systemctl --user start hoop
journalctl --user -u hoop -f
```

9. **On the NEW host: Restart NEEDLE workers**

```bash
# This depends on your NEEDLE setup
# Example: cd ~/project-name && needle fleet start
```

**Pitfalls:**
- **Project paths:** If project paths differ between old and new host, update `projects.yaml` before starting HOOP.
- **Missing workspaces:** HOOP restore does NOT migrate project code or `.beads/` directories. You must do this separately.
- **Tailscale IPs:** If HOOP's listen address is bound to a specific Tailscale IP that changed, update `config.yml`.
- **NEEDLE worker state:** Workers are NOT migrated by HOOP. They need to be restarted on the new host and will reconnect to their existing `.beads/` state.

### Rollback on failed restore

All four scenarios use the same rollback mechanism. If `hoop restore` fails mid-operation:

1. Original state is preserved at `~/.hoop.rollback.YYYYMMDDTHHMMSSZ`
2. Automatic rollback restores the original `~/.hoop/` directory
3. Manual recovery is possible if automatic rollback also fails:

```bash
# If restore failed and automatic rollback also failed:
mv ~/.hoop.rollback.YYYYMMDDTHHMMSSZ ~/.hoop
```

## Manual backup/restore (without S3)

For simple setups without S3 backup configured, you can manually copy the database:

### Database

The primary database is `~/.hoop/fleet.db`. It contains:

- Audit log with hash chain
- Stitch metadata
- Agent sessions
- Reflection ledger

For backup:

```bash
cp ~/.hoop/fleet.db ~/.hoop/fleet.db.backup.$(date +%Y%m%d)
```

For restore:

```bash
systemctl --user stop hoop
cp ~/.hoop/fleet.db.backup.YYYYMMDD ~/.hoop/fleet.db
systemctl --user start hoop
```

### Project registry

The project registry is stored in `~/.hoop/projects.yaml`. Back up alongside the database if you have custom project configurations.

## Uninstalling

```bash
# Stop and disable the service
systemctl --user stop hoop
systemctl --user disable hoop

# Remove the service file
rm ~/.config/systemd/user/hoop.service
systemctl --user daemon-reload

# Remove HOOP data (optional)
rm -rf ~/.hoop

# Remove the binary
rm ~/.local/bin/hoop
```
