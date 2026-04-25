# HOOP Troubleshooting Guide

This guide helps diagnose and recover from common HOOP issues.

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

1. `/debug/state` output: `curl http://127.0.0.1:3000/debug/state`
2. Recent logs: `journalctl --user -u hoop -n 200`
3. Daemon version: from `version.daemon` in `/debug/state`
4. Schema version: from `version.schema` in `/debug/state`

This information helps diagnose the issue quickly.
