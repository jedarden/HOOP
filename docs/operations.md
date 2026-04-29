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

## Release Playbook

This section documents the end-to-end release process for HOOP, from version bump to post-release verification.

> **Note:** The full CI/CD pipeline (GitHub releases with binaries) is planned but not yet implemented. Currently, only Docker image builds are automated via `hoop-build.yaml`. See "Current State" below.

### Prerequisites

- **GitHub access:** Write access to `jedarden/HOOP` (for releases and manual binary uploads)
- **Argo Workflows access:** `iad-ci` cluster kubeconfig (for monitoring CI)
- **Docker Hub access:** `ronaldraygun/hoop` repository (automated via `docker-hub-registry` secret)
- **Workspace:** Clean git working directory (no uncommitted changes)

### Current State

The following automation exists today:

| Workflow | Status | What it does |
|----------|--------|--------------|
| `hoop-build` | ✅ Implemented | Docker build → `ronaldraygun/hoop:latest` |
| `hoop-ci` | ❌ Not implemented | Planned: CI checks + binary build + GitHub Release |

Until `hoop-ci` is implemented, binary releases must be done manually (see "Manual Binary Release" below).

### Release Process

#### 1. Update CHANGELOG

Move all entries from `[Unreleased]` to the new version section. Follow [SemVer](https://semver.org/) for version bump kind:

- **MAJOR** (`1.0.0` → `2.0.0`): Breaking changes, no backwards compatibility
- **MINOR** (`1.0.0` → `1.1.0`): Additive/backwards-compatible changes
- **PATCH** (`1.0.0` → `1.0.1`): Bug fixes without schema shape changes

```bash
# Edit CHANGELOG.md
vim CHANGELOG.md

# Change header from:
## [Unreleased]

# To:
## [0.2.0] — 2026-04-29

# Create new [Unreleased] section at top
```

#### 2. Bump versions

HOOP uses a workspace version scheme. Update both Rust and Node.js versions:

```bash
# Bump Cargo.toml workspace version
vim Cargo.toml
# Change: version = "0.1.0" → version = "0.2.0"

# Bump hoop-ui/web/package.json version
vim hoop-ui/web/package.json
# Change: "version": "0.1.0" → "0.2.0"
```

#### 3. Commit and tag

```bash
# Stage changes
git add CHANGELOG.md Cargo.toml hoop-ui/web/package.json

# Commit with version message
git commit -m "Release v0.2.0"

# Create annotated tag
git tag -a v0.2.0 -m "HOOP v0.2.0"

# Push commit and tag
git push origin main
git push origin v0.2.0
```

#### 4. Build Docker image (automated)

The `hoop-build` WorkflowTemplate in `iad-ci` builds and pushes the Docker image:

```bash
# Manually trigger the build (or wait for webhook trigger)
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-build-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-build
  arguments:
    parameters:
      - name: image_tag
        value: latest
EOF

# Watch for workflow creation
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflows -n argo-workflows -w | grep hoop-build

# Stream logs from build step
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig logs -n argo-workflows <pod-name> -c build-image -f
```

**Expected workflow behavior:**
1. Clones HOOP from GitHub
2. Uses Kaniko to build Docker image from `Dockerfile`
3. Pushes `ronaldraygun/hoop:latest` to Docker Hub

**Troubleshooting:**

```bash
# Check workflow phase
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflow <name> -n argo-workflows -o jsonpath='{.status.phase} - {.status.message}'

# Get per-node failure details
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig get workflow <name> -n argo-workflows -o json | python3 -c "
import json,sys
w = json.load(sys.stdin)
for node in w['status'].get('nodes',{}).values():
    if node.get('phase') in ('Failed','Error'):
        print(node['displayName'], '-', node['phase'])
        print('  msg:', node.get('message',''))
"
```

#### 5. Create GitHub Release with binaries (manual)

Until `hoop-ci` is implemented, binary releases are manual:

```bash
# Build release binaries locally
cargo build --profile --release -p hoop-cli -p hoop-mcp
strip target/release/hoop target/release/hoop-mcp

# Create GitHub Release
gh release create v0.2.0 \
  --repo jedarden/HOOP \
  --title "HOOP v0.2.0" \
  --notes "See CHANGELOG.md for details" \
  target/release/hoop \
  target/release/hoop-mcp
```

#### 6. Verify Docker Hub image

#### 5. Verify GitHub Release

```bash
# Check release exists
gh release view v0.2.0 --repo jedarden/HOOP

# Verify artifacts
gh release view v0.2.0 --repo jedarden/HOOP --json assets,name,url

# Expected assets:
# - hoop (linux-x86_64 binary)
# - hoop-mcp (linux-x86_64 binary)
```

#### 6. Verify Docker Hub image

```bash
# Check image was pushed
docker pull ronaldraygun/hoop:latest

# Verify image metadata
docker inspect ronaldraygun/hoop:latest | jq '.[0].Created'

# Run smoke test
docker run --rm ronaldraygun/hoop:latest hoop --version
```

#### 7. Post-release verification

Test installation on a fresh host to ensure release artifacts work:

```bash
# Download and install binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop \
  -o /tmp/hoop && chmod +x /tmp/hoop

# Verify version
/tmp/hoop --version

# Test daemon startup (dry-run)
/tmp/hoop --help

# Test Docker image
docker run --rm -p 3000:3000 ronaldraygun/hoop:latest
```

**Verification checklist:**
- [ ] Binary downloads and executes
- [ ] `hoop --version` reports correct version
- [ ] `hoop --help` displays usage
- [ ] Docker image pulls and runs
- [ ] CHANGELOG.md entry is complete and accurate
- [ ] No `[Unreleased]` entries remain (except new section header)

### Manual Binary Release

Until `hoop-ci` is implemented, binary releases require manual building and uploading:

```bash
# 1. Build release binaries
cargo build --release -p hoop-cli -p hoop-mcp

# 2. Strip debug symbols (reduces size by ~80%)
strip target/release/hoop target/release/hoop-mcp

# 3. Verify binaries work
./target/release/hoop --version
./target/release/hoop-mcp --version

# 4. Create GitHub Release with binaries attached
gh release create v0.2.0 \
  --repo jedarden/HOOP \
  --title "HOOP v0.2.0" \
  --notes "Release notes:

$(awk '/## \[0\.2\.0\]/,/^## / {print}' CHANGELOG.md | head -n -1)
" \
  target/release/hoop#hoop-linux-x86_64 \
  target/release/hoop-mcp#hoop-mcp-linux-x86_64
```

**Planned automation:** The future `hoop-ci` workflow will automate this process, including:
- CI checks (`fmt`, `clippy`, `test`)
- Cross-platform binary builds
- Automatic GitHub Release creation
- Triggered on tag push via GitHub webhook

### Rollback Procedure

If a release has critical issues, follow the rollback process:

#### Option 1: Hotfix patch release (preferred)

Fix the issue in main and release a new patch version:

```bash
# 1. Fix the bug
vim <affected-files>

# 2. Bump to patch version (e.g., 0.2.0 → 0.2.1)
vim Cargo.toml
vim hoop-ui/web/package.json
vim CHANGELOG.md  # Document fix under new [0.2.1] section

# 3. Commit and tag
git add -A
git commit -m "Hotfix: fix critical issue in v0.2.0"
git tag -a v0.2.1 -m "HOOP v0.2.1 (hotfix)"
git push origin main
git push origin v0.2.1

# 4. Build and push new Docker image
kubectl --kubeconfig=/home/coding/.kube/iad-ci.kubeconfig create -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: hoop-build-
  namespace: argo-workflows
spec:
  workflowTemplateRef:
    name: hoop-build
EOF

# 5. Create new GitHub Release with fixed binaries
cargo build --release -p hoop-cli -p hoop-mcp
strip target/release/hoop target/release/hoop-mcp
gh release create v0.2.1 --repo jedarden/HOOP \
  --title "HOOP v0.2.1 (hotfix)" \
  --notes "Hotfix for critical issue in v0.2.0" \
  target/release/hoop target/release/hoop-mcp
```

#### Option 2: Delete release and tag (emergency only)

Use this only if the release is broken and cannot be fixed with a patch:

```bash
# 1. Delete the GitHub release
gh release delete v0.2.0 --repo jedarden/HOOP --yes

# 2. Delete the tag locally and remotely
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# 3. Fix the issue, bump version again, and re-release
```

**Pitfalls:**
- **Binary users:** Users who already installed the broken release will need to reinstall. Document the hotfix release prominently in CHANGELOG.
- **Docker users:** `ronaldraygun/hoop:latest` will be fixed on next release. Tagged versions (e.g., `ronaldraygun/hoop:v0.2.0`) cannot be deleted from Docker Hub—push a new image to the same tag if needed.
- **Cached artifacts:** GitHub CDN caches releases. Allow up to 30 minutes for propagation before declaring rollback complete.

### Dry-Run Release Test

Before cutting a real release, test the process end-to-end with a dummy version:

```bash
# 1. Create a test branch
git checkout -b release-dry-run

# 2. Bump to test version (not to be released)
vim Cargo.toml  # Change: version = "0.0.0-test"
vim hoop-ui/web/package.json  # Change: "version": "0.0.0-test"

# 3. Commit and tag
git add Cargo.toml hoop-ui/web/package.json
git commit -m "Test release (dry-run)"
git tag -a v0.0.0-test -m "HOOP v0.0.0-test (dry-run)"

# 4. Test local binary build
cargo build --release -p hoop-cli -p hoop-mcp
./target/release/hoop --version  # Should report 0.0.0-test

# 5. Test GitHub Release creation (dry-run mode)
gh release create v0.0.0-test \
  --repo jedarden/HOOP \
  --title "DRY-RUN: HOOP v0.0.0-test" \
  --notes "This is a test release. Delete after verification." \
  --draft \
  target/release/hoop#hoop-linux-x86_64

# 6. Verify release looks correct
gh release view v0.0.0-test --repo jedarden/HOOP

# 7. Cleanup after test
git checkout main
git branch -D release-dry-run
git tag -d v0.0.0-test
gh release delete v0.0.0-test --repo jedarden/HOOP --yes
```

### Version Scheme Reference

HOOP follows [Semantic Versioning 2.0.0](https://semver.org/):

| Kind | Example | When to use |
|------|---------|-------------|
| MAJOR | `1.0.0` → `2.0.0` | Breaking schema changes, removed APIs, incompatible migrations |
| MINOR | `1.0.0` → `1.1.0` | New features, additive schema fields, backwards-compatible changes |
| PATCH | `1.0.0` → `1.0.1` | Bug fixes, performance improvements, no schema shape changes |

**Current version:** `0.1.0` (initial development release—breaking changes may occur before `1.0.0`)

### Planned Automation: hoop-ci Workflow

The `hoop-ci` WorkflowTemplate is planned but not yet implemented. When complete, it will automate the full release process on tag push:

**Expected workflow behavior:**
1. Clone HOOP from trusted source (ignoring webhook payload for security)
2. Install Rust toolchain + gh CLI
3. Build web UI (`hoop-ui/web`)
4. Run CI checks: `cargo fmt --check`, `cargo clippy`, `cargo test`
5. Extract version from `Cargo.toml` workspace
6. Build release binaries (`hoop`, `hoop-mcp`) for `x86_64-unknown-linux-gnu`
7. Create GitHub Release with binaries attached
8. Push Docker image to `ronaldraygun/hoop:latest`

**Trigger:** GitHub webhook on tag push (`v*.*.*`)

**To implement:** Create `.argo/workflowtemplates/hoop-ci.yaml` modeled after existing CI workflows (e.g., `needle-ci`, `forge-ci` in `declarative-config`)

**Current version:** `0.1.0` (initial development release—breaking changes may occur before `1.0.0`)
