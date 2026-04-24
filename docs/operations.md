# HOOP Operations Guide

This guide covers the operational aspects of running HOOP in production.

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
| `TimeoutStartSec` | `30` | Give daemon 30s to start |
| `TimeoutStopSec` | `30` | Give daemon 30s to stop gracefully |

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

### Manual installation

If you need to customize the service unit, copy the template and edit:

```bash
mkdir -p ~/.config/systemd/user
cp /path/to/hoop.service.template ~/.config/systemd/user/hoop.service
# Edit the file as needed
systemctl --user daemon-reload
systemctl --user enable hoop
systemctl --user start hoop
```

## Upgrading

```bash
# 1. Pull the new binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Restart the service
systemctl --user restart hoop
```

State in `~/.hoop/` persists across upgrades. Schema migrations run on startup.

## Backup and restore

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
