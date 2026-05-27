# HOOP Quickstart

This guide will get you up and running with HOOP in under 5 minutes.

## Try with Docker (quickest way to explore)

The Docker image is the fastest way to try HOOP without installing anything on your host.

```bash
# Pull and run HOOP
docker run -d \
  --name hoop \
  -p 3000:3000 \
  -v hoop-data:/root/.hoop \
  ronaldraygun/hoop:latest

# Open http://localhost:3000 in your browser
```

**Volume mounts for persistence:**

| Mount | Purpose |
|-------|---------|
| `/root/.hoop` | SQLite database, config, projects registry |

**Custom configuration via environment variables:**

```bash
docker run -d \
  --name hoop \
  -p 3000:3000 \
  -v hoop-data:/root/.hoop \
  -e HOOP_BIND_ADDR=0.0.0.0:3000 \
  -e HOOP_AGENT_ADAPTER=claude \
  -e HOOP_AGENT_MODEL=claude-sonnet-4-20250514 \
  -e HOOP_AGENT_ANTHROPIC_API_KEY=sk-ant-... \
  ronaldraygun/hoop:latest
```

**Available environment variables:**

- `HOOP_BIND_ADDR` - Server bind address (default: `127.0.0.1:3000`)
- `HOOP_AGENT_ADAPTER` - LLM adapter: `claude`, `codex`, `opencode`, `gemini`, `aider`
- `HOOP_AGENT_MODEL` - Model name (adapter-specific)
- `HOOP_AGENT_ANTHROPIC_API_KEY` - Anthropic API key (for Claude adapter)
- `HOOP_ZAI_BASE_URL` - ZAI proxy URL (optional)
- `HOOP_ZAI_API_KEY` - ZAI API key (optional)
- `HOOP_RATE_LIMIT_RPM` - Rate limit in requests per minute
- `HOOP_COST_CAP_USD` - Cost cap in USD
- `HOOP_METRICS_ENABLED` - Enable Prometheus metrics (`true`/`false`)
- `HOOP_METRICS_PORT` - Metrics port (default: `9091`)

## Native install (recommended for production)

For a production setup on a long-lived host, install the binary directly:

```bash
# 1. Pull the binary
curl -sSL https://github.com/jedarden/HOOP/releases/latest/download/hoop-linux-x86_64 \
  -o ~/.local/bin/hoop && chmod +x ~/.local/bin/hoop

# 2. Run the first-time wizard
hoop init
```

`hoop init` walks you through:

1. **Dependency check** — verifies `br`, `tmux`, Tailscale membership, port availability
2. **Project registration** — scan and register your workspaces
3. **Agent setup** (optional) — configure LLM adapter
4. **systemd install** — auto-start on boot
5. **Health check** — confirms HOOP is running and prints access URLs

At the end of the wizard, HOOP prints:

- **Local URL:** `http://localhost:3000` (always available)
- **Tailscale URL:** `http://<tailscale-hostname>:3000` (if Tailscale is installed and logged in)

If Tailscale is not detected, the wizard prints instructions for enabling Tailscale access.

## First five minutes

Once HOOP is running:

1. **Open the dashboard** at `http://localhost:3000`
2. **Register a project** via `hoop projects add ~/path/to/project`
3. **Start a conversation** with the human-interface agent
4. **Dictate a note** using voice input (hotkey: `Ctrl+Shift+V`)

## Next steps

- See [`docs/operations.md`](operations.md) for systemd setup, backups, troubleshooting
- See [`AGENTS.md`](../AGENTS.md) for contributor guidelines
- See [`docs/plan/plan.md`](plan/plan.md) for the full implementation plan
