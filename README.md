# 🪡 HOOP

**Control plane for NEEDLE fleets — launches, tensions, and steers the work.**

A single Rust daemon that runs on your coding host and serves as the human-facing interface to a multi-project NEEDLE worker fleet. Answers questions about what your workers are doing and drafts new work when you ask — without ever managing the workers themselves.

> ⚠️ **Status — 2026-04:** HOOP is pre-v0.1. Implementation has not started. This README describes the target install flow; treat it as a preview of how things will work when v0.1 ships. See [`docs/plan/plan.md`](docs/plan/plan.md) for the current state, the full roadmap, and every design decision in detail.

---

## 📖 Documentation map

| File | Who it's for | What's in it |
|---|---|---|
| [`README.md`](README.md) (this file) | Humans | Quickstart — install, concepts, first five minutes |
| [`AGENTS.md`](AGENTS.md) | LLM contributors | Repository guide, terminology, non-goals, conventions |
| [`CHANGELOG.md`](CHANGELOG.md) | Everyone | Version history following Keep-a-Changelog / SemVer |
| [`docs/operations.md`](docs/operations.md) | Operators | Systemd service, logs, upgrades, backup, troubleshooting |
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

```bash
# 1. Pull the binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Run the first-time wizard
hoop init
```

`hoop init` walks you through:

1. ✅ **Dependency check** — verifies `br`, `tmux`, each configured CLI adapter, Tailscale membership, port availability, disk room. Any failure is reported with the exact command to fix it.
2. 📁 **Project registration** — offers `scan ~/` with a preview of every directory containing a `.beads/`. You pick which ones to register, give each a friendly name, and you're done.
3. 🤖 **Agent setup (optional)** — asks for Anthropic credentials if you want the human-interface agent enabled. Skippable; can enable later.
4. 🔧 **systemd install** — writes `~/.config/systemd/user/hoop.service` with auto-restart on failure (max 5 restarts per 5min).
5. 🌐 **Health check + URL** — confirms HOOP is running, prints the Tailscale URL you can open in a browser.

Total time: under 5 minutes if your tools are already installed.

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

For more operational details, see [`docs/operations.md`](docs/operations.md).

---

## 🧭 Where to go next

- 🔧 [`docs/operations.md`](docs/operations.md) — systemd service management, logs, upgrades, backup, and troubleshooting.
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

## 🤝 For contributors

If you're reading this because you want to help build HOOP:

- 📖 Read [`AGENTS.md`](AGENTS.md) — it's the LLM-facing version of this document and covers repo conventions.
- 🪜 The plan's phased roadmap is strict — don't start phase N+1 work before phase N meets its success criteria.
- 🔤 Match the terminology exactly. "Mayor" / "polecat" / "swarm" / "Gas Town" vocabulary was removed from earlier drafts; do not re-introduce it.
- 🚫 Non-goals are not suggestions — they're the design. HOOP never steers workers, never enforces capacity, never mutates bead state beyond `br create`.

---

## 🧶 Sibling projects in the NEEDLE ecosystem

- 🔗 **[`dicklesworthstone/beads_rust`](https://github.com/dicklesworthstone/beads_rust)** — `br`, the bead queue. HOOP depends on it; shells out to it.
- 🧵 **[`jedarden/NEEDLE`](https://github.com/jedarden/NEEDLE)** — the worker supervision system. HOOP observes NEEDLE's events and writes beads NEEDLE workers pick up.
- 🧵 **[`jedarden/FABRIC`](https://github.com/jedarden/FABRIC)** — passive read-only observability. HOOP links to FABRIC via a URL bridge.

Each tool has one job. Together they form the operator's view of a long-running multi-agent coding fleet.
