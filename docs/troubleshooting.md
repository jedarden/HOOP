# HOOP Troubleshooting Guide

This guide helps diagnose and recover from common HOOP issues, with mappings to `hoop audit` output for quick identification.

## Quick diagnostic: `hoop audit`

The `hoop audit check` command performs a comprehensive runtime audit and reports issues with fix commands. It should be your first diagnostic step.

```bash
# Run the full audit
hoop audit check

# Output includes checks for:
# - br (beads_rust) installation and version
# - tmux availability
# - git version
# - Tailscale membership
# - systemd service status
# - disk space
# - port availability
# - project paths validity
```

### Audit output mapping

Each audit check reports:
- **Check name** - What's being verified
- **Status** - ✅ passed, ❌ critical failure, ⚠️ warning
- **Description** - What the check found
- **Fix command** - Exact command to resolve the issue

### Common audit failures and recovery

| Check Name | Failure Symptom | Fix Command | Recovery Steps |
|------------|----------------|-------------|---------------|
| `br_installed` | `br not found in PATH` | `cargo install --git https://github.com/dicklesworthstone/beads_rust br` | Install beads_rust |
| `br_version` | `br version too old` | `cargo install --git https://github.com/dicklesworthstone/beads_rust br --force` | Update beads_rust |
| `tmux_installed` | `tmux not found` | `sudo apt install tmux` | Install tmux |
| `git_version` | `git too old (need 2.5+)` | `sudo apt install git` | Update git |
| `tailscale_status` | `Not connected to Tailscale` | `tailscale up` | Connect to Tailscale |
| `systemd_service` | `hoop.service not found` | `hoop install-systemd` | Install systemd service |
| `disk_space` | `Less than 1GB free` | `clean up disk space` | Free up disk space |
| `port_available` | `Port 3000 already in use` | `lsof -i :3000` to find process | Stop conflicting process |

### Audit severity levels

- **Critical** - Daemon will not start. Must fix before running `hoop serve`.
- **Warning** - Daemon will start with degraded features. Fix for full functionality.
- **Info** - Informational only. No action required.

## Quick diagnostic: `/debug/state`

The `GET /debug/state` endpoint returns a complete snapshot of the daemon's runtime state for incident triage. It's available at `http://127.0.0.1:3000/debug/state` when the daemon is running.

### Using the endpoint

```bash
# Get the full state snapshot
curl http://127.0.0.1:3000/debug/state | jq

# Check specific sections
curl http://127.0.0.1:3000/debug/state | jq '.workers'
curl http://127.0.0.1:3000/debug/state | jq '.active_claims'
curl http://127.0.0.1:3000/debug/state | jq '.backup_timestamps'
```

### Response fields

| Field | Description |
|-------|-------------|
| `schema_version` | JSON schema version (bumped on field changes) |
| `uptime_secs` | Seconds since daemon started |
| `version.daemon` | HOOP daemon version |
| `version.schema` | Schema version |
| `config_hash` | SHA-256 of resolved config (detects config changes) |
| `bind_addr` | Address daemon is bound to |
| `workers` | Fleet roster with state, liveness, last heartbeat, PID |
| `worker_pids` | All observed worker PIDs |
| `active_claims` | Workers currently executing beads |
| `ws_clients` | Active WebSocket connections |
| `session_alias_table` | CLI session ID → worker/bead mappings |
| `backup_timestamps` | Last successful backup time and size |
| `fleet_db_path` | Path to fleet.db |
| `fleet_db_size_bytes` | Size of fleet.db file |
| `fleet_db_wal_size_bytes` | Size of fleet.db WAL file |
| `open_stitches` | Count of open beads |
| `total_beads` | Total beads known to daemon |
| `projects` | List of project names |

### Common diagnostic queries

```bash
# Check for hung or dead workers
curl http://127.0.0.1:3000/debug/state | jq '.workers[] | select(.liveness != "Live")'

# Check for stuck workers (knot state)
curl http://127.0.0.1:3000/debug/state | jq '.workers[] | select(.state | startswith("Knot"))'

# Verify backup is recent
curl http://127.0.0.1:3000/debug/state | jq '.backup_timestamps.last_success_iso'

# Check database file size (large WAL may need checkpoint)
curl http://127.0.0.1:3000/debug_state | jq '.fleet_db_wal_size_bytes'
```

## Worker issues

### Worker shows "Hung" or "Dead" liveness

**Symptom:** Worker in `/debug/state` shows `liveness: "Hung"` or `"Dead"`

**Cause:** Worker process stopped sending heartbeats (likely crashed or hung)

**Diagnosis:**
```bash
# Check worker state
curl http://127.0.0.1:3000/debug/state | jq '.workers[] | select(.worker == "worker-name")'

# Check if process is still running
ps aux | grep -i python | grep -v grep
```

**Recovery:**
1. Check logs: `journalctl --user -u hoop -n 100`
2. Restart the daemon: `systemctl --user restart hoop`
3. Workers will auto-reconnect when the CLI runs again

### Worker stuck in "Knot" state

**Symptom:** Worker shows `state: "Knot { reason: "..." }"`

**Cause:** Worker encountered an error it couldn't recover from

**Diagnosis:**
```bash
# Get the reason
curl http://127.0.0.1:3000/debug/state | jq '.workers[] | select(.state | startswith("Knot"))'
```

**Recovery:**
1. The reason field explains what went wrong
2. Check the CLI session logs for the affected bead
3. Fix the underlying issue (permissions, missing files, etc.)
4. Restart the daemon or close/reopen the stuck bead

## Database issues

### Large WAL file

**Symptom:** `fleet_db_wal_size_bytes` is large relative to `fleet_db_size_bytes`

**Cause:** Many writes without checkpoint (WAL grows until checkpoint)

**Diagnosis:**
```bash
# Check WAL size
curl http://127.0.0.1:3000/debug/state | jq '.fleet_db_wal_size_bytes'
```

**Recovery:**
1. Restart the daemon (triggers checkpoint on shutdown)
2. WAL is automatically checkpointed during graceful shutdown

### Database corruption

**Symptom:** Queries fail with "database disk image is malformed"

**Diagnosis:**
```bash
# Check integrity
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"
```

**Recovery:**
1. Restore from backup: `cp ~/.hoop/fleet.db.backup.YYYYMMDD ~/.hoop/fleet.db`
2. If no backup, export and reimport (last resort)

## Backup issues

### Backup not running

**Symptom:** `backup_timestamps.last_success_iso` is old or `null`

**Diagnosis:**
```bash
# Check last backup
curl http://127.0.0.1:3000/debug/state | jq '.backup_timestamps'

# Check backup config
cat ~/.hoop/config.yml | grep -A 10 backup:
```

**Recovery:**
1. Verify backup configuration in `~/.hoop/config.yml`
2. Check S3 credentials are set: `env | grep AWS_`
3. Check daemon logs: `journalctl --user -u hoop -n 100`

## WebSocket issues

### WS clients not connecting

**Symptom:** UI shows "disconnected" or `/debug/state` shows empty `ws_clients`

**Diagnosis:**
```bash
# Check active WS connections
curl http://127.0.0.1:3000/debug/state | jq '.ws_clients'

# Check if daemon is listening
lsof -i :3000
```

**Recovery:**
1. Verify daemon is running: `systemctl --user status hoop`
2. Check browser console for WebSocket errors
3. Restart the daemon

## Configuration issues

### Config not reloading

**Symptom:** Changes to `~/.hoop/config.yml` don't take effect

**Diagnosis:**
```bash
# Get current config hash
curl http://127.0.0.1:3000/debug/state | jq '.config_hash'

# Check for reload errors in logs
journalctl --user -u hoop --since "5 minutes ago" | grep -i config
```

**Recovery:**
1. Validate config YAML syntax
2. Restart the daemon: `systemctl --user restart hoop`
3. Check logs for validation errors

## Performance issues

### High memory usage

**Diagnosis:**
```bash
# Check process memory
curl http://127.0.0.1:3000/debug/state | jq '.fleet_db_size_bytes'

# Check open beads (many open beads increase memory)
curl http://127.0.0.1:3000/debug/state | jq '.open_stitches'
```

**Recovery:**
1. Close old beads to free memory
2. Consider periodic bead cleanup
3. Check for memory leaks if issue persists

### Slow response times

**Diagnosis:**
1. Check metrics endpoint: `curl http://127.0.0.1:3000/metrics`
2. Look for high request duration: `hoop_http_request_duration_ms`

**Recovery:**
1. Check disk I/O (slow disk affects SQLite)
2. Reduce number of concurrent workers
3. Check for lock contention in database

## Getting help

When reporting issues, include:

1. `hoop audit check` output: `hoop audit check`
2. `/debug/state` output: `curl http://127.0.0.1:3000/debug/state`
3. Recent logs: `journalctl --user -u hoop -n 200`
4. Daemon version: from `version.daemon` in `/debug/state`
5. Schema version: from `version.schema` in `/debug/state`

This information helps diagnose the issue quickly.

## Common failures mapped to `hoop audit` output

### Dependency failures

#### `br_installed` check fails

**Audit output:**
```
❌ br_installed
   br (beads_rust) is required for bead operations
   Fix: cargo install --git https://github.com/dicklesworthstone/beads_rust br
```

**Recovery:**
```bash
# Install beads_rust
cargo install --git https://github.com/dicklesworthstone/beads_rust br

# Verify installation
br --version

# Re-run audit
hoop audit check
```

#### `br_version` check fails

**Audit output:**
```
❌ br_version
   br version is too old (need 0.1.28+, have 0.1.0)
   Fix: cargo install --git https://github.com/dicklesworthstone/beads_rust br --force
```

**Recovery:**
```bash
# Force reinstall to get latest version
cargo install --git https://github.com/dicklesworthstone/beads_rust br --force

# Verify version
br --version

# Re-run audit
hoop audit check
```

#### `tmux_installed` check fails

**Audit output:**
```
❌ tmux_installed
   tmux is required for observing NEEDLE workers
   Fix: sudo apt install tmux
```

**Recovery:**
```bash
# Debian/Ubuntu
sudo apt install tmux

# Fedora/RHEL
sudo dnf install tmux

# Verify
tmux -V
```

#### `git_version` check fails

**Audit output:**
```
❌ git_version
   git version is too old (need 2.5+, have 2.0.0)
   Fix: sudo apt install git
```

**Recovery:**
```bash
# Update git
sudo apt install git

# Verify version
git --version
```

### Environment failures

#### `tailscale_status` check fails

**Audit output:**
```
⚠️  tailscale_status
   Not connected to Tailscale (optional but recommended)
   Fix: tailscale up
```

**Recovery:**
```bash
# Connect to Tailscale
sudo tailscale up

# Verify connection
tailscale status

# Get Tailscale IP
tailscale ip -4
```

#### `systemd_service` check fails

**Audit output:**
```
⚠️  systemd_service
   hoop.service not found in ~/.config/systemd/user/
   Fix: hoop install-systemd
```

**Recovery:**
```bash
# Install systemd service
hoop install-systemd

# Enable and start
systemctl --user daemon-reload
systemctl --user enable hoop
systemctl --user start hoop
```

#### `disk_space` check fails

**Audit output:**
```
❌ disk_space
   Less than 1GB free on /home partition
   Fix: clean up disk space
```

**Recovery:**
```bash
# Check disk usage
df -h ~/

# Find large files
du -sh ~/.hoop/*

# Clean up if needed
# - Remove old backups
# - Clear attachment cache
# - Clean journal logs
sudo journalctl --vacuum-size=500M
```

#### `port_available` check fails

**Audit output:**
```
❌ port_available
   Port 3000 is already in use
   Fix: lsof -i :3000 to find process
```

**Recovery:**
```bash
# Find process using port 3000
lsof -i :3000

# Stop the conflicting process
kill <PID>

# Or change HOOP's port in config.yml
vim ~/.hoop/config.yml
# Change: bind_addr: "127.0.0.1:3001"
```

### Project path failures

#### `project_path_exists` check fails

**Audit output:**
```
❌ project_path_exists
   Project path does not exist: /path/to/project
   Fix: hoop projects remove <name> && hoop projects add /correct/path
```

**Recovery:**
```bash
# Remove invalid project
hoop projects remove <project-name>

# Re-add with correct path
hoop projects add /correct/path --name <project-name>

# Verify
hoop projects list
```

#### `project_has_beads` check fails (warning)

**Audit output:**
```
⚠️  project_has_beads
   Project /path/to/project has no .beads/ directory
   Fix: Initialize beads with: br init
```

**Recovery:**
```bash
# Navigate to project
cd /path/to/project

# Initialize beads
br init

# Verify
ls .beads/
```

## Startup failures

### Daemon won't start

**Symptoms:**
- `systemctl --user start hoop` fails immediately
- Service enters failed state
- No logs in journal

**Diagnosis:**
```bash
# Check service status
systemctl --user status hoop

# Check logs
journalctl --user -u hoop -n 50

# Run audit
hoop audit check
```

**Recovery:**
1. Fix any critical audit failures
2. Verify dependencies are installed
3. Check port availability
4. Ensure disk space is sufficient
5. Restart service: `systemctl --user restart hoop`

### Daemon crashes on startup

**Symptoms:**
- Service starts but immediately exits
- Logs show panic or assertion failure
- Repeated restart failures

**Diagnosis:**
```bash
# Get crash details
journalctl --user -u hoop -n 100 | grep -i panic

# Check database integrity
sqlite3 ~/.hoop/fleet.db "PRAGMA integrity_check;"

# Verify config file syntax
cat ~/.hoop/config.yml
```

**Recovery:**
1. If database corrupted: restore from backup
2. If config invalid: fix YAML syntax
3. If missing directory: `mkdir -p ~/.hoop`
4. Check for version mismatch: `hoop --version`

## Migration failures

### Schema migration fails on startup

**Symptoms:**
- Daemon starts but exits with migration error
- Logs show "migration failed" or "schema version mismatch"
- `hoop migrate status` shows pending migrations

**Diagnosis:**
```bash
# Check migration status
hoop migrate status

# View current schema version
sqlite3 ~/.hoop/fleet.db "SELECT value FROM metadata WHERE key = 'schema_version';"

# Check logs for error details
journalctl --user -u hoop -n 100 | grep -i migration
```

**Recovery:**
```bash
# 1. Ensure you have a backup
cp ~/.hoop/fleet.db ~/.hoop/fleet.db.backup.$(date +%Y%m%d)

# 2. Run migrations manually
hoop migrate run --confirm

# 3. If migration fails, rollback
hoop migrate rollback <previous-version> --confirm

# 4. If rollback fails, restore from backup
cp ~/.hoop/fleet.db.backup.YYYYMMDD ~/.hoop/fleet.db
```

### Major version upgrade blocked

**Symptoms:**
- `hoop migrate run` fails with "major upgrade required"
- Daemon refuses to start with version mismatch

**Diagnosis:**
```bash
# Check versions
hoop --version
hoop migrate status

# Look for major version gate error
journalctl --user -u hoop -n 50 | grep -i "major.*upgrade"
```

**Recovery:**
```bash
# Run major upgrade
hoop migrate major-upgrade --confirm

# This performs one-way migration
# Cannot rollback after major upgrade
# Ensure backup exists first!
```

## Backup failures

### Backup not running

**Symptoms:**
- `backup_timestamps.last_success_iso` is old or null
- No backup logs in journal
- S3 sync not happening

**Diagnosis:**
```bash
# Check last backup
curl http://127.0.0.1:3000/debug/state | jq '.backup_timestamps'

# Check backup config
cat ~/.hoop/config.yml | grep -A 10 backup:

# Check for credential errors
journalctl --user -u hoop --since today | grep -i backup
```

**Recovery:**
1. Verify backup configuration in `~/.hoop/config.yml`
2. Check S3 credentials are set: `env | grep AWS_`
3. Test S3 access manually
4. Trigger manual backup: `curl -X POST http://localhost:3000/api/backup/trigger`

### Restore fails mid-operation

**Symptoms:**
- `hoop restore` fails with partial download
- Rollback directories left behind
- Daemon refuses to start after failed restore

**Diagnosis:**
```bash
# Check for rollback directories
ls -la ~/.hoop.rollback.*

# Check restore logs
journalctl --user -u hoop -n 100 | grep -i restore

# Verify S3 credentials
env | grep HOOP_BACKUP
```

**Recovery:**
```bash
# 1. Clean up rollback directories
rm -rf ~/.hoop.rollback.*

# 2. Restore from original state
mv ~/.hoop.rollback.YYYYMMDDTHHMMSSZ ~/.hoop

# 3. Verify S3 credentials and retry
export HOOP_BACKUP_ENDPOINT="..."
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# 4. Retry restore
hoop restore --from s3://bucket/prefix/snapshot-id
```

## Getting help (detailed)

When reporting issues, include the following information:

### 1. Audit output

```bash
hoop audit check > ~/hoop-audit.txt
```

### 2. Debug state

```bash
curl http://127.0.0.1:3000/debug/state > ~/hoop-debug-state.json
```

### 3. Recent logs

```bash
journalctl --user -u hoop -n 200 > ~/hoop-logs.txt
```

### 4. Version information

```bash
hoop --version > ~/hoop-version.txt
```

### 5. Schema version

```bash
hoop migrate status --json > ~/hoop-migration-status.json
```

Attach all five files when reporting issues for fastest diagnosis.

1. `/debug/state` output: `curl http://127.0.0.1:3000/debug/state`
2. Recent logs: `journalctl --user -u hoop -n 200`
3. Daemon version: from `version.daemon` in `/debug/state`
4. Schema version: from `version.schema` in `/debug/state`

This information helps diagnose the issue quickly.
