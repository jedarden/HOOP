# 🪡 HOOP

**Control plane for NEEDLE fleets — launches, tensions, and steers the work.**

A single Rust daemon that runs on your coding host and serves as the human-facing interface to a multi-project NEEDLE worker fleet. Answers questions about what your workers are doing and drafts new work when you ask — without ever managing the workers themselves.

> **v1.0.0 Now Available** — Production-ready control plane for NEEDLE fleets. See [RELEASE_NOTES_v1.0.md](RELEASE_NOTES_v1.0.md) for what's new in this release.

---

## 📖 Documentation map

| File | Who it's for | What's in it |
|---|---|---|
| [`README.md`](README.md) (this file) | Humans | Quickstart — install, concepts, first five minutes |
| [`AGENTS.md`](AGENTS.md) | LLM contributors | Repository guide, terminology, non-goals, conventions |
| [`CHANGELOG.md`](CHANGELOG.md) | Everyone | Version history following Keep-a-Changelog / SemVer |
| [`docs/operations.md`](docs/operations.md) | Operators | Systemd service, logs, upgrades, backups, migrations, Tailscale routing |
| [`docs/troubleshooting.md`](docs/troubleshooting.md) | Operators | Common failures mapped to `hoop audit` output, recovery steps |
| [`docs/plan/plan.md`](docs/plan/plan.md) | Everyone going deep | **The canonical implementation plan.** 13 sections covering vision, principles, architecture, data flows, phased roadmap (v0.1 → v1.0), marquee capabilities, technology decisions, non-goals, open questions, milestones, onboarding, and a Kubernetes appendix. Your first stop after this README. |
| [`docs/notes/`](docs/notes/) | Contributors & LLMs | Prior-art research: feature inventory, architecture patterns, interop with NEEDLE, orchestrator problems and solutions |

---

## 🎯 What HOOP is, in one paragraph

HOOP is a single Rust daemon you run on your coding host. It watches every NEEDLE fleet and every headless-CLI conversation across every repo on the host, gives you one web UI to see all of it, and provides a conversational agent that can answer questions about your work and draft new work when you ask. HOOP does not run or control NEEDLE's workers — they live on their own. HOOP is the pane of glass and the handle.

### 🚫 What HOOP is not

- Not a worker orchestrator — NEEDLE does that
- Not a bead mutator beyond creation — the only write HOOP performs is `br create`
- Not a capacity enforcer — it shows utilization, never throttles or rotates
- Not a strand router — strands are worker-immutable, set at launch by (model, harness)
- Not a replacement for FABRIC — FABRIC is passive read-only observability; HOOP is local-host with one write
- Not multi-host — one HOOP, one host; growth means more projects, not more hosts

---

## 📋 Prerequisites

You'll need:

- 🖥️ **A long-lived Linux host on Tailscale** (or equivalent private network). An EX44-class machine is the baseline target; smaller hosts work with fewer concurrent projects.
- 🔗 **`br` installed** — [beads_rust by Jeffrey Emanuel](https://github.com/dicklesworthstone/beads_rust). HOOP shells out to `br` for all bead operations.
- 🖼️ **`tmux`** — HOOP doesn't spawn tmux sessions, but it observes NEEDLE workers running in tmux.
- 📚 **`git` 2.5+** — for worktree inspection in the file browser.
- 💬 **At least one headless CLI** installed and credentialed in its native cache: Claude Code, Codex, OpenCode, Gemini, or Aider. HOOP never touches their credentials; it only reads their session logs.
- 🧵 **NEEDLE** (optional for initial install) — HOOP runs in read-only mode without a NEEDLE fleet, though most features are more useful with one.
- 🔑 **Anthropic API key or Claude Code account** (optional for initial install) — needed when you enable the human-interface agent in phase 5.

---

## 📦 Install

### Quick install (latest release)

```bash
# 1. Pull the v1.0.0 binary
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Verify installation
hoop --version
# hoop 1.0.0

# 3. Run the first-time wizard
hoop init
```

### Install from source

```bash
# Clone the repository
git clone https://github.com/jedarden/HOOP.git
cd HOOP

# Build release binary
cargo build --release

# Install to PATH
sudo cp target/release/hoop /usr/local/bin/
# or for user-only install:
cp target/release/hoop ~/.local/bin/

# Run first-time setup
hoop init
```

### Requirements

| Tool | Minimum version | Recommended version | Install |
|------|----------------|-------------------|---------|
| `br` (beads_rust) | 0.1.28 | 0.1.28+ | `cargo install --git https://github.com/dicklesworthstone/beads_rust` |
| `git` | 2.5+ | 2.47+ | System package manager (`apt install git` / `dnf install git`) |
| `tmux` | 3.0+ | 3.5a+ | System package manager (`apt install tmux` / `dnf install tmux`) |
| Rust | 1.75+ | 1.83+ (stable) | `rustup.rs` (for building from source only) |

**Verified versions:** HOOP v1.0.0 is tested and verified against `br` 0.1.28, `git` 2.47.3, `tmux` 3.5a, and Rust 1.95.0. Earlier versions may work but are not actively tested.

### First-time setup walkthrough

`hoop init` walks you through:

1. ✅ **Dependency check** — verifies `br`, `tmux`, each configured CLI adapter, Tailscale membership, port availability, disk room. Any failure is reported with the exact command to fix it.
2. 📁 **Project registration** — offers `scan ~/` with a preview of every directory containing a `.beads/`. You pick which ones to register, give each a friendly name, and you're done.
3. 🤖 **Agent setup (optional)** — asks for Anthropic credentials if you want the human-interface agent enabled. Skippable; can enable later.
4. 🔧 **systemd install** — writes `~/.config/systemd/user/hoop.service` with auto-restart on failure (max 5 restarts per 5min).
5. 🌐 **Health check + URL** — confirms HOOP is running, prints the Tailscale URL you can open in a browser.

Total time: under 5 minutes if your tools are already installed.

---

## ⚡ Quick Start: Up and Running in 10 Minutes

Follow this step-by-step guide to get HOOP running with the testrepo workspace in under 10 minutes.

### Step 1: Install HOOP (2 minutes)

```bash
# Pull the v1.0.0 binary
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# Verify installation
hoop --version
# Expected output: hoop 1.0.0
```

**If you don't have the prerequisites installed:**

```bash
# Install br (beads_rust) - required for bead operations
cargo install --git https://github.com/dicklesworthstone/beads_rust

# Install tmux (for observing NEEDLE workers)
sudo apt install tmux  # Debian/Ubuntu
# or: sudo dnf install tmux  # Fedora/RHEL

# Install git (if not already installed)
sudo apt install git  # Debian/Ubuntu
```

### Step 2: Run First-Time Setup (2 minutes)

```bash
# Start the setup wizard
hoop init
```

The wizard will:
1. Check dependencies (`br`, `tmux`, `git`)
2. Scan for projects with `.beads/` directories
3. Offer to register projects (you can skip this and use testrepo)
4. Set up the agent (optional — select "Skip" for now)
5. Install the systemd service
6. Start the daemon and provide the URL

### Step 3: Register the Testrepo (1 minute)

```bash
# Register the included testrepo workspace
hoop projects add /home/coding/HOOP/testrepo --name testrepo

# Verify registration
hoop projects list
# Expected output:
# Registered projects:
#   - testrepo (1 workspace)
```

The testrepo contains:
- **Pre-populated beads** — Synthetic open, claimed, closed, and failed beads
- **CLI session fixtures** — Example Claude, Codex, OpenCode, Gemini, and Aider sessions
- **Attachments** — Test images, audio, video, and log files
- **Source code** — ~500 synthetic Rust files for file browser testing

### Step 4: Open the Web UI (1 minute)

```bash
# Get the URL
hoop url
# Expected output:
# http://localhost:3000
# or http://100.x.y.z:3000 (if on Tailscale)
```

Open the URL in your browser. You should see:
- A dashboard with the testrepo project card
- Synthetic Stitches showing different states (open, claimed, closed)
- File browser for exploring the testrepo source code

### Step 5: Explore the Interface (4 minutes)

**Dashboard (home page):**
- Project cards showing active work, cost today, and alerts
- Click on testrepo to see the Stitch timeline

**Project Detail (click testrepo card):**
- Stitch list showing all conversations in the project
- Filter by status: open, claimed, closed, failed
- Click on any Stitch to see details

**File Browser:**
- Navigate through testrepo source code
- Syntax highlighting for Rust files
- File tree on the left, code viewer on the right

### Step 6: Verify Service Status (optional)

```bash
# Check HOOP is running
hoop status
# Expected output:
# HOOP daemon is running (v1.0.0)
#    PID: 12345
#    Uptime: 2 minutes

# Check systemd service
systemctl --user status hoop
# Expected output: active (running)
```

### Next Steps

Now that HOOP is running:

1. **Add your own projects:**
   ```bash
   hoop projects add /path/to/your/project --name myproject
   ```

2. **Enable the agent (optional):**
   ```bash
   hoop agent setup
   # Follow prompts to enter Anthropic API key
   ```

3. **Configure backup (optional):**
   ```bash
   # Edit ~/.hoop/config.yml
   # Add backup configuration for S3-compatible storage
   ```

4. **Set up ADB dictation (optional):**
   ```bash
   ./scripts/hoop-adb setup
   # Follow prompts for Pixel 6 integration
   ```

---

## 🔗 Full Installation Examples

### Verified installation example (testrepo)

HOOP includes a synthetic test workspace at `testrepo/` that you can use to verify your installation in under 10 minutes:

```bash
# 1. Install HOOP (if not already installed)
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Run first-time setup (select "Skip" for agent setup to keep it simple)
hoop init

# 3. Register the testrepo project
hoop projects add /home/coding/HOOP/testrepo --name testrepo

# 4. Verify HOOP sees the testrepo data
hoop projects list
# → Registered projects:
#   - testrepo (1 workspace)

# 5. Open the web UI and explore
echo "Open this URL in your browser:"
hoop url
# → http://localhost:3000

# 6. In the UI, verify you see:
#    - testrepo project card with synthetic Stitches
#    - Stitch list showing open/claimed/closed beads
#    - File browser for testrepo source code
```

The testrepo contains:
- **Pre-populated beads** — Synthetic open, claimed, closed, and failed beads for testing
- **CLI session fixtures** — Example Claude, Codex, OpenCode, Gemini, and Aider sessions
- **Attachments** — Test images, audio, video, and log files
- **Source code** — ~500 synthetic Rust files for file browser testing

This lets you explore HOOP's UI and features without setting up a real NEEDLE fleet or waiting for LLM work to complete.

---

## 🧵 Concepts cheat sheet

You'll encounter these terms in the UI:

| Term | What it means |
|---|---|
| **🗂️ Project** | A logical unit you care about — may contain one or more repos (workspaces). You control the list. |
| **📂 Workspace** | A single repo on disk with its own `.beads/` queue. A project can span multiple workspaces. |
| **🪡 Stitch** | A single conversation inside a project. Types: operator chat, dictated voice note, NEEDLE worker session, ad-hoc CLI session. |
| **🧩 Pattern** | An optional grouping of Stitches around a goal. Can span projects — good for epics and long-running initiatives. |
| **🔸 Bead** | NEEDLE's internal execution unit. You rarely need to see these; HOOP abstracts them into Stitches for you. |
| **🤖 Human-interface agent** | A persistent Claude Code session HOOP hosts. Your primary conversation partner; answers questions, drafts Stitches. |
| **📖 Reflection Ledger** | HOOP's learned-rules store. When you repeat an instruction across Stitches, the agent proposes a durable rule you can approve. |

You don't need to know what a bead is to use HOOP. You work in Stitches.

---

## 🚀 First five minutes in the UI

1. 📊 **Open the dashboard.** You'll see one card per project, aggregating active work, cost today, and any alerts.
2. 🔍 **Click into a project.** The Stitch list shows every conversation that's happened there — worker sessions from your NEEDLE fleet, any ad-hoc `claude` sessions you've run in that repo, and any operator chats with the agent.
3. 🎤 **Dictate a note.** Press the hotkey (or the mic button on your phone if you've set up ADB). Talk for 30 seconds about something you're thinking about. When you stop, a dictated Stitch appears in that project's timeline with audio + transcript.
4. 💬 **Ask the agent something.** Open the chat pane. Try `what's going on in <project>?` — the agent summarizes active Stitches, recent failures, and cost trends.
5. 📄 **Browse a file.** Open the file browser for the project. Hover any line in a code file; once Stitch-Provenance lands (phase 2), you'll see which Stitch last modified it.

---

## 📸 Screenshots

<div align="center">

### Project Dashboard
![Project Dashboard](docs/screenshots/dashboard.png)
*One card per project, aggregating active work, cost today, and alerts.*

### Stitch Timeline
![Stitch Timeline](docs/screenshots/project-detail.png)
*All conversations in a project — worker sessions, operator chats, dictated notes.*

### Agent Chat
![Agent Chat](docs/screenshots/agent-chat.png)
*Ask questions, draft work, get summaries — your primary interface to HOOP.*

### File Browser
![File Browser](docs/screenshots/file-browser.png)
*Navigate project files with code syntax highlighting and Stitch-aware change tracking.*

</div>

> **Note:** Screenshots show anonymized data from the testrepo workspace. For live demos with your own projects, run `hoop init` and open the provided URL.

**See your own interface in under 10 minutes:**

```bash
# Install HOOP
HOOP_VERSION="1.0.0"
curl -sSL "https://github.com/jedarden/HOOP/releases/download/v${HOOP_VERSION}/hoop-linux-x86_64" \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# Run first-time setup (select "Skip" for agent setup to keep it simple)
hoop init

# Register the testrepo project
hoop projects add /home/coding/HOOP/testrepo --name testrepo

# Open the web UI
echo "Open this URL in your browser:"
hoop url
# → http://localhost:3000
#    or http://100.x.y.z:3000 (Tailscale)
```

---

## ✅ Quick-start verification

Verify your installation in under 5 minutes:

```bash
# 1. Check HOOP is running
hoop status
# → HOOP daemon is running (v1.0.0)
#    PID: 12345
#    Uptime: 2 minutes

# 2. Check projects are registered
hoop projects list
# → Registered projects:
#   - HOOP (1 workspace)
#   - kalshi-weather (1 workspace)

# 3. Open the web UI
echo "Open this URL in your browser:"
hoop url
# → http://localhost:3000
#    or http://100.x.y.z:3000 (Tailscale)

# 4. Verify you can see Stitches
# In the UI: click on any project card → should see Stitch list
# Each project shows: active Stitches, cost today, any alerts

# 5. Test the agent (if enabled)
# In the UI: open chat pane → ask "what projects are registered?"
# Agent should respond with your project list and summaries

# 6. Check service status
systemctl --user status hoop
# Should show: active (running)
```

If any step fails, see the [troubleshooting section](#-troubleshooting) below.

---

## ✍️ Creating your first work

The agent can draft work for you. Try:

> "Investigate why the kalshi-weather rate-limit retries are failing more often in the evening window."

The agent will:

1. 🔎 Read relevant conversations, logs, and recent Stitches in that project.
2. 📝 Propose a Stitch draft — title, description, any needed attachments.
3. 💰 Show you a preview with estimated cost, duration, and risk assessment.
4. ✅ On your confirm, create the necessary beads in the right workspace with a `stitch:<id>` label so HOOP can track the work.

NEEDLE workers will pick up the beads on their own schedule. Watch the Stitch in the project view — it'll show worker Stitches spawning under it as the work progresses.

---

## 📱 Pixel 6 ADB dictation (push-to-talk)

Capture voice notes from your Pixel 6 using ADB over Tailscale — no Android app needed beyond Termux.

### How it works

```
Pixel 6 (Termux)                   Tailscale               Coding host
────────────────                   ─────────               ────────────
[mic] → recording.m4a   ──POST /api/adb/dictate──►  HOOP daemon
termux-microphone-record              (raw bytes)    ├─ store audio
                                                     ├─ create stitch
                                                     └─ enqueue Whisper
```

1. `hoop-adb start [project]` broadcasts `HOOP_DICTATE_START` to the phone via ADB
2. The Termux listener records audio using `termux-microphone-record`
3. `hoop-adb stop` broadcasts `HOOP_DICTATE_STOP` — listener stops and uploads via curl
4. HOOP creates a dictated note in the active project (or the one specified)
5. Whisper transcribes the audio asynchronously

### Phone setup

```bash
# 1. Run the setup guide from your coding host
./scripts/hoop-adb setup

# 2. Push the listener script to the phone
adb push scripts/termux-hoop-listener.sh /data/data/com.termux/files/home/hoop-listener.sh
adb shell chmod +x /data/data/com.termux/files/home/hoop-listener.sh

# 3. Inside Termux on the phone, install deps and start listener
pkg install termux-api sox curl
# Edit ~/hoop-listener.sh: set HOOP_URL to your Tailscale IP (e.g. http://100.x.y.z:3000)
nohup ~/hoop-listener.sh > ~/.hoop-listener.log 2>&1 &
```

Termux and Termux:API must be installed from **F-Droid** (not Google Play). Grant
microphone permission to Termux in Android Settings → Apps → Termux → Permissions.

### Usage

```bash
# Start recording (associate with a specific project)
hoop-adb start HOOP

# Start recording (uses whatever project you last navigated to in the UI)
hoop-adb start

# Stop recording — listener uploads automatically
hoop-adb stop

# Check which project HOOP will file notes under
hoop-adb status
```

### Active-project API

The UI automatically calls `PUT /api/ui/active-project` when you navigate to a project
card, so the ADB endpoint knows where to file notes without a `?project=` parameter.

You can also POST audio directly (useful for scripting or CI):

```bash
# Direct upload with explicit project
curl -X POST "http://localhost:3000/api/adb/dictate?project=HOOP&filename=note.m4a" \
     --data-binary @recording.m4a \
     -H "Content-Type: audio/mp4"
```

### Troubleshooting

| Symptom | Fix |
|---|---|
| `adb: no devices` | Run `adb-check` on the coding host; reconnect with `adb-connect <port>` |
| `No active project` error | Navigate to a project in the UI first, or pass `?project=name` |
| Note appears but transcript stuck at "Pending" | Whisper model not at `~/.hoop/models/ggml-base.en.bin` |
| Upload fails (HTTP 000) | Check Tailscale is up; verify `HOOP_URL` in the Termux listener script |

---

## ☀️ Daily rhythm (once v0.5 lands)

After HOOP has been running for a few days, the agent will produce a **Morning Brief** when you log in:

- ✅ What closed overnight, ❌ what failed (with cost impact), ⚠️ what's stuck, 📈 what's anomalous
- 📋 Pre-drafted Stitches for follow-ups it thinks are important (always preview — nothing auto-submitted)
- ⭐ One headline: the single thing it'd prioritize today, with evidence

You skim it, accept or redirect the drafts, and you've got your day planned in two minutes.

---

## 📚 Adding more projects

```bash
hoop projects add /path/to/new/repo
# or to re-scan
hoop projects scan ~/
```

### 🔗 Multi-repo projects

If several repos compose one logical unit (a migration project spanning source + config + secrets), register them together:

```bash
hoop projects add-multi kalshi-weather-migration \
  /home/coding/kalshi-weather:source \
  /home/coding/declarative-config:manifests \
  /home/coding/apexalgo-iad-secrets:secrets
```

---

## ⬆️ Upgrade flow

```bash
# 1. Pull the new binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Restart
systemctl --user restart hoop
```

State in `~/.hoop/` persists across upgrades. Schema migrations run on startup; a daily snapshot of `fleet.db` gives you a rollback point.

---

## 🔧 Configuration Examples

HOOP includes example configuration files for common use cases in [`docs/examples/`](docs/examples/). Copy these to `~/.hoop/` and customize for your environment.

### Quick Setup with Examples

```bash
# Create config directory
mkdir -p ~/.hoop

# Copy example configurations
cp docs/examples/config.yml ~/.hoop/
cp docs/examples/accounts.yaml ~/.hoop/

# Edit to customize (optional)
nano ~/.hoop/config.yml
```

### Example: Minimal Local-Only Setup

Perfect for single-developer workflows with no network exposure:

```yaml
# ~/.hoop/config.yml
server:
  bind_addr: "127.0.0.1:3000"  # Localhost only

ui:
  theme: dark
  default_project_sort: activity

agent:
  model: claude-sonnet-4-6
  morning_brief_enabled: true
```

### Example: Tailscale-Exposed with Backup

For multi-host access via Tailscale with automated backups:

```yaml
# ~/.hoop/config.yml
server:
  bind_addr: "0.0.0.0:3000"  # Expose on all interfaces

backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-yourname
  prefix: hoop/
  schedule: "0 4 * * *"  # Daily at 4 AM
  retention_days: 30
  encryption: true  # Set HOOP_BACKUP_AGE_KEY env var
```

### Example: Multi-Account Rate Limits

Configure rate limits for multiple Claude Code accounts:

```yaml
# ~/.hoop/accounts.yaml
accounts:
  claude-code-default:
    adapter: claude-code
    limits:
      prompts_per_5h: 1600
      prompts_per_7d: 8000
      tokens_per_minute: 40000

  claude-code-build:
    adapter: claude-code
    limits:
      prompts_per_5h: 500
      prompts_per_7d: 2000
      tokens_per_minute: 20000
```

### Example: Multi-Repo Project

Track a deployment spanning multiple repositories:

```bash
hoop projects add-multi myservice-deployment \
  /home/coding/myservice:source \
  /home/coding/declarative-config:manifests \
  /home/coding/secrets:secrets
```

Or configure directly in `~/.hoop/projects.yaml`:

```yaml
projects:
  - name: myservice-deployment
    description: My service deployment across repos
    workspaces:
      - path: /home/coding/myservice
        role: source
      - path: /home/coding/declarative-config
        role: manifests
      - path: /home/coding/secrets
        role: secrets
```

### Example: NEEDLE Fleet Configuration

Configure a NEEDLE worker fleet (HOOP observes but doesn't control):

```yaml
# ~/.needle/fleet.yaml (in each NEEDLE workspace)
name: example-fleet
workspace: /home/coding/myproject

workers:
  - name: worker-opus
    model: claude-opus-4-7
    harness: claude-code
    concurrency: 1

  - name: worker-sonnet
    model: claude-sonnet-4-6
    harness: claude-code
    concurrency: 2

cost:
  pricing:
    claude-opus-4-7:
      input: 15.0
      output: 75.0
      cache_read: 0.3
      cache_write: 3.75
    claude-sonnet-4-6:
      input: 3.0
      output: 15.0
      cache_read: 0.06
      cache_write: 0.30
```

### Common Configuration Patterns

| Use Case | Key Settings | Description |
|----------|-------------|-------------|
| **Local development** | `bind_addr: "127.0.0.1:3000"` | No network exposure |
| **Tailscale access** | `bind_addr: "0.0.0.0:3000"` | Expose on Tailscale interface |
| **Automated backup** | `backup.enabled: true` | Daily S3 backups |
| **High-volume tier** | `accounts.limits.prompts_per_5h: 1600` | Claude Max limits |
| **Multi-repo project** | `projects.add-multi` | Group related repos |
| **Cost monitoring** | `pricing.per_million` | Track per-model costs |

For more examples, see [`docs/examples/README.md`](docs/examples/README.md).

---

## 🔧 Advanced Configuration Patterns

Run HOOP locally without network exposure:

```yaml
# ~/.hoop/config.yml
server:
  bind_addr: "127.0.0.1:3000"

ui:
  theme: dark

agent:
  model: claude-sonnet-4-6
  morning_brief_enabled: true

backup:
  enabled: false
```

### Pattern 2: Tailscale-exposed with backup

Expose on Tailscale interface with automated backups:

```yaml
# ~/.hoop/config.yml
server:
  bind_addr: "0.0.0.0:3000"

backup:
  endpoint: https://s3.us-west-000.backblazeb2.com
  bucket: hoop-backups-yourname
  schedule: "0 4 * * *"
  retention_days: 30
  encryption: true
```

Set `HOOP_BACKUP_AGE_KEY` environment variable with your age public key.

### Pattern 3: Multi-repo project

Track a deployment spanning multiple repos:

```bash
hoop projects add-multi myservice-deployment \
  /home/coding/myservice:source \
  /home/coding/declarative-config:manifests \
  /home/coding/secrets:secrets
```

### Pattern 4: High-volume Claude Max tier

Configure rate limits for Claude Max:

```yaml
# ~/.hoop/accounts.yaml
accounts:
  claude-code-default:
    adapter: claude-code
    limits:
      prompts_per_5h: 1600
      prompts_per_7d: 8000
      tokens_per_minute: 40000
```

---

## 📁 Example configurations

Example configuration files are available in [`docs/examples/`](docs/examples/):

| File | Purpose |
|------|---------|
| [`config.yml`](docs/examples/config.yml) | UI preferences, backup settings, agent configuration |
| [`accounts.yaml`](docs/examples/accounts.yaml) | Per-adapter rate limits and account settings |
| [`projects.yaml`](docs/examples/projects.yaml) | Project registry examples (single-repo, multi-repo, migration) |
| [`fleet.yaml`](docs/examples/fleet.yaml) | NEEDLE worker fleet configuration |

See [`docs/examples/README.md`](docs/examples/README.md) for detailed usage patterns and common configuration scenarios.

Copy these to `~/.hoop/` and customize for your environment:

```bash
# Create config directory
mkdir -p ~/.hoop

# Copy and customize examples
cp docs/examples/config.yml ~/.hoop/
cp docs/examples/accounts.yaml ~/.hoop/
# Edit ~/.hoop/config.yml to set your preferences
```

---

## 💥 When HOOP dies

Nothing else notices. NEEDLE keeps running. FABRIC keeps working. Your CLIs keep writing session files. The next time you start HOOP it rebuilds its view entirely from disk. HOOP is a convenience, not a dependency.

---

## 🔧 Troubleshooting

| 🚨 Symptom | 🔎 First check |
|---|---|
| `hoop init` fails at dependency check | Run the command it suggests; re-run `hoop init` |
| Web UI won't load | `systemctl --user status hoop`; Tailscale up; correct hostname |
| Project shows an error card | That project's `.beads/` moved or got corrupted; `hoop projects list --verbose` |
| Stitches show "unknown adapter" | CLI adapter config missing or binary not in PATH |
| Agent won't respond | Anthropic key not set, or rate limit hit — check the capacity widget |
| Morning Brief empty | Needs at least a few closed operator Stitches to have material; try again tomorrow |

For more operational details, see [`docs/operations.md`](docs/operations.md) and [`docs/troubleshooting.md`](docs/troubleshooting.md).

---

## 🧭 Where to go next

- 🔧 [`docs/operations.md`](docs/operations.md) — systemd service management, logs, upgrades, backups, migrations, Tailscale routing.
- 🔧 [`docs/troubleshooting.md`](docs/troubleshooting.md) — common failures mapped to `hoop audit` output, recovery steps.
- 📘 [`docs/plan/plan.md`](docs/plan/plan.md) — the full implementation plan. Your next read if you want to understand *why* HOOP is shaped the way it is. Covers:
  - §1 Vision, §1.5 Roles, §1.6 Hierarchy (Pattern → Stitch → Bead)
  - §2 Environment, §2.1 The `br` dependency
  - §3 Principles (13 of them, all load-bearing)
  - §4 Component architecture (daemon, project registry, per-project runtime, Stitch/Pattern/Reflection services)
  - §5 Data flows (single-project reader, bead creation, ad-hoc vs fleet classification)
  - §6 Phased roadmap (v0.1 → v1.0, seven phases)
  - §6.5 Marquee capabilities summary (13 features that earn HOOP its keep)
  - §7 Technology decisions
  - §8 Non-goals (explicit, 12 of them)
  - §9 Open questions
  - §10 Milestones
  - §11 Relationship diagram
  - §12 Onboarding & documentation
  - §13 Kubernetes appendix (deferred)
- 📂 [`docs/notes/`](docs/notes/) — prior-art research that shaped the design:
  - Reference feature inventory
  - Architecture patterns worth absorbing
  - Interop with NEEDLE
  - Orchestrator problems and solutions (field survey + 12 "prevent by design" rules)
- 📊 `/metrics` endpoint (phase 6) — Prometheus-format fleet / cost / capacity metrics
- 🔍 `/debug/state` endpoint (phase 6) — runtime introspection for incident triage

---

## 🤝 Contributing

We welcome contributions! HOOP is a Rust + TypeScript project with a focus on reliability and operator experience.

### Development setup

```bash
# Clone the repository
git clone https://github.com/jedarden/HOOP.git
cd HOOP

# Install Rust dependencies
cargo build

# Install UI dependencies
cd hoop-ui/web
pnpm install
cd ../..

# Run tests
cargo test

# Run with hot-reload during development
cargo run --bin hoop -- serve --dev

# Run UI in development mode (separate terminal)
cd hoop-ui/web && pnpm dev
```

### Prerequisites for development

| Tool | Minimum version | Purpose |
|------|----------------|---------|
| Rust | 1.83+ (stable) | Daemon build |
| Node.js | 20+ | UI development |
| pnpm | 9+ | UI package manager |
| br (beads_rust) | 0.1.28+ | Bead operations |
| just | 0.12+ (optional) | Task runner |

### Contribution guidelines

1. **Read [`AGENTS.md`](AGENTS.md)** — Repository conventions, terminology, and LLM collaboration patterns
2. **Follow the phased roadmap** — Don't start phase N+1 work before phase N meets its success criteria (see [`docs/plan/plan.md`](docs/plan/plan.md) §6)
3. **Match terminology exactly** — Use "Stitch", "Pattern", "workspace", not "ticket", "epic", "repo"
4. **Respect non-goals** — HOOP never steers workers, never enforces capacity, never mutates bead state beyond `br create`
5. **Test your changes** — Run the full test suite including integration tests
6. **Document schema changes** — Update CHANGELOG.md for any schema modifications
7. **Follow Rust conventions** — Use `cargo fmt`, `cargo clippy`, and respect Rust idioms
8. **Respect TypeScript conventions** — Use the existing ESLint config and type checking

### Pull request process

1. Fork and create a feature branch from `main`
2. Make your changes with tests
3. Run `cargo fmt` and `cargo clippy` to ensure code quality
4. Update CHANGELOG.md if applicable
5. Submit PR with description linking to relevant beads/issues
6. CI will run tests, schema drift check, and performance budget verification

### Code review criteria

PRs are reviewed based on:
- **Correctness** — Does the change do what it claims? Tests must pass.
- **Clarity** — Is the code readable and well-documented where needed?
- **Consistency** — Does it match existing patterns and conventions?
- **Performance** — Does it respect performance budgets? (UI: Core Web Vitals, daemon: latency targets)
- **Completeness** — Are docs, tests, and CHANGELOG updated?

### Areas seeking contribution

- **Mobile UI** — Responsive improvements for phone form factor
- **Additional adapters** — Support for more CLI tools (Cursor, Windsurf, etc.)
- **Reflection rules** — New rule types and learning patterns
- **Morning Brief** — Enhanced summarization and draft quality
- **Documentation** — Screenshots, demo videos, tutorials
- **Performance** — Query optimization, caching strategies
- **Testing** — Integration test coverage, load testing scenarios

### Development workflow

```bash
# 1. Create a feature branch
git checkout -b feature/my-feature

# 2. Make your changes
# ... edit files ...

# 3. Run tests locally
cargo test
cd hoop-ui/web && pnpm test && pnpm lint

# 4. Check formatting
cargo fmt --check
cargo clippy -- -D warnings

# 5. Commit with conventional commits
git commit -m "feat: add support for Cursor adapter"

# 6. Push and create PR
git push origin feature/my-feature
# Create PR on GitHub
```

### Testing with testrepo

Use the included testrepo for verification:

```bash
# Register testrepo for local testing
hoop projects add /home/coding/HOOP/testrepo --name testrepo

# Verify UI shows synthetic beads
# Open http://localhost:3000 and check testrepo project
```

### Release process

Releases are automated via Argo Workflows:

1. Update version in `Cargo.toml` and `hoop-ui/web/package.json`
2. Update CHANGELOG.md with release notes
3. Commit and tag: `git tag v1.x.x`
4. Push to trigger CI/CD: `git push origin v1.x.x`
5. GitHub Release is created automatically with binary attachments

### Code of conduct

Be respectful, constructive, and focused on the work. We're building tools for operators — empathy for the user experience is our north star.

### Getting help

- **Documentation** — Start with [AGENTS.md](AGENTS.md) and [docs/plan/plan.md](docs/plan/plan.md)
- **Issues** — Search existing issues before creating new ones
- **Discussions** — Use GitHub Discussions for questions and ideas
- **Pull requests** — Draft PRs are welcome for early feedback

---

## 🧶 Sibling projects in the NEEDLE ecosystem

- 🔗 **[`dicklesworthstone/beads_rust`](https://github.com/dicklesworthstone/beads_rust)** — `br`, the bead queue. HOOP depends on it; shells out to it.
- 🧵 **[`jedarden/NEEDLE`](https://github.com/jedarden/NEEDLE)** — the worker supervision system. HOOP observes NEEDLE's events and writes beads NEEDLE workers pick up.
- 🧵 **[`jedarden/FABRIC`](https://github.com/jedarden/FABRIC)** — passive read-only observability. HOOP links to FABRIC via a URL bridge.

Each tool has one job. Together they form the operator's view of a long-running multi-agent coding fleet.
